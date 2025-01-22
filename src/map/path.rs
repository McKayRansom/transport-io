use building::BuildingType;
use pathfinding::prelude::astar;
use tile::{Reservation, Tile, YieldType};
use vehicle::SPEED_TICKS_PER_TILE;

use super::*;

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;

impl Grid {
    pub fn reserve(
        &mut self,
        pos: &Position,
        vehicle_id: Id,
    ) -> Result<Reservation, ReservationError> {
        self.get_tile_mut(pos)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(vehicle_id, *pos)
    }

    pub fn find_path(&self, start: (Position, Direction), end: &Id) -> Path {
        let path_start = start.0 + start.1;

        if !self.get_tile(&path_start)?.is_road() {
            return None;
        }

        let end_building = self.buildings.hash_map.get(end)?;
        if matches!(end_building.building_type, BuildingType::House) {
            // append destination pos (this is already checked to be a road)
            let end_pos_dir = end_building.destination_pos(&self)?;
            let end_pos = end_pos_dir.0 + end_pos_dir.1;
            if let Some(mut path) = self.find_road_path(&path_start, &end_pos) {
                path.0.push(end_pos_dir.0);
                path.1 += 1;
                Some(path)
            } else {
                None
            }
        } else {
            self.find_path_to_building(&path_start, end, &end_building.pos)
        }
    }

    pub fn find_road_path(&self, start: &Position, end: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end) / 3,
            |p| p == end,
        )
    }

    pub fn find_path_to_building(&self, start: &Position, end: &Id, end_approx: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end_approx) / 3,
            |p| {
                if let Some(tile) = self.get_tile(p) {
                    tile.get_building_id() == Some(*end)
                } else {
                    false
                }
            },
        )
    }

    pub fn road_successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        let tile = self.get_tile(pos);
        if tile.is_none() {
            return Vec::new();
        }
        let tile = tile.unwrap();
        tile.iter_connections(pos)
            .iter()
            .filter_map(|dir| {
                let new_pos = *pos + *dir;
                self.get_tile(&new_pos).map(|tile| (new_pos, tile.cost()))
            })
            .collect()
    }

    pub fn should_be_yielded_to(
        &self,
        should_yield: YieldType,
        pos: &Position,
        dir_from: Direction,
    ) -> bool {
        self.get_tile(pos)
            .is_some_and(|tile| tile.should_be_yielded_to(should_yield, dir_from))
    }

    pub fn should_we_yield_when_entering(
        &self,
        should_yield: YieldType,
        position: &Position,
    ) -> Option<Position> {
        // never yield from an intersection
        if should_yield == YieldType::Never {
            return None;
        }

        if let Some(Tile::Road(road)) = self.get_tile(position) {
            // For each direction that feeds into this tile in question
            for dir in Direction::ALL
                .iter()
                .filter(|&dir| !road.is_connected(*dir))
            {
                let yield_to_pos = *position + *dir;
                if self.should_be_yielded_to(should_yield, &yield_to_pos, *dir) {
                    return Some(yield_to_pos);
                }
            }
        }

        None
    }
}

pub enum ReservePathError {
    InvalidPath,
    Blocking(Position),
}

#[derive(Serialize, Deserialize)]
pub struct VehiclePath {
    pub grid_path: Path,
    pub reserved: Vec<Reservation>,
    path_index: usize,
    path_time_ticks: u32,
    elapsed_ticks: u32,
    pub destination: Id,

    // This is an optimization and doesn't need to be saved
    #[serde(skip_serializing, skip_deserializing)]
    pub blocking_tile: Option<Position>,
}

impl VehiclePath {
    pub fn new(
        id: Id,
        grid: &mut Grid,
        start: (Position, Direction),
        destination: Id,
    ) -> Result<Self, ReservationError> {
        let reservation = grid
            .get_tile_mut(&start.0)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(id, start.0)?;

        Ok(Self {
            grid_path: None,
            path_time_ticks: 0,
            path_index: 0,
            destination,
            blocking_tile: None,
            elapsed_ticks: 0,
            reserved: vec![reservation],
        })
    }

    pub fn fixup(&mut self, id: Id, grid: &mut Grid) -> Result<(), ReservationError> {
        // Fix serialization
        for reservation in &mut self.reserved {
            *reservation = grid.reserve(&reservation.pos, id)?
        }
        Ok(())
    }

    fn reserve_path(
        &mut self,
        id: Id,
        current_pos: Position,
        grid: &mut Grid,
    ) -> Result<Vec<Reservation>, ReservePathError> {
        // TODO: Move to grid

        let should_yield = grid
            .get_tile(&current_pos)
            .ok_or(ReservePathError::InvalidPath)?
            .should_yield();

        if self.grid_path.is_none() {
            return Err(ReservePathError::InvalidPath);
        }

        let mut reserved = Vec::<Reservation>::new();

        // for pos in &path[self.path_index + 1..] {
        if let Some(pos) = self
            .grid_path
            .as_ref()
            .unwrap()
            .0
            .get(self.path_index)
            .cloned()
        {
            // TODO Make function
            match grid.reserve(&pos, id) {
                Ok(reservation) => {
                    reserved.push(reservation);
                    if let Some(yield_to_pos) =
                        grid.should_we_yield_when_entering(should_yield, &pos)
                    {
                        return Err(ReservePathError::Blocking(yield_to_pos));
                    }
                    // Fall through
                }
                Err(ReservationError::TileInvalid) => {
                    return Err(ReservePathError::InvalidPath);
                }
                Err(ReservationError::TileReserved) => {
                    return Err(ReservePathError::Blocking(pos));
                }
            }
        }

        Ok(reserved)
    }

    pub fn find_path(&mut self, grid: &mut Grid, start: (Position, Direction)) -> bool {
        self.grid_path = grid.find_path(start, &self.destination);

        if let Some(path) = &self.grid_path {
            self.path_time_ticks = path.1 * SPEED_TICKS_PER_TILE as u32;
            self.path_index = 0;
        }
        self.grid_path.is_some()
    }

    pub fn get_next_pos(
        &mut self,
        id: Id,
        grid: &mut Grid,
        start: (Position, Direction),
    ) -> Option<Position> {
        match self.reserve_path(id, start.0, grid) {
            Ok(reserved) => {
                self.reserved = reserved;
                self.path_index += 1;
                self.reserved.first().map(|reservation| reservation.pos)
            }
            Err(ReservePathError::InvalidPath) => {
                self.find_path(grid, start);
                None
            }
            Err(ReservePathError::Blocking(blocking_pos)) => {
                self.blocking_tile = Some(blocking_pos);
                None
            }
        }
    }

    pub fn update_trip(&mut self) {
        self.elapsed_ticks += 1;
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.grid_path {
            let tiles_elapsed =
                (self.elapsed_ticks.saturating_sub(1) / SPEED_TICKS_PER_TILE as u32) + 1;
            let tiles_expected = path.1;

            let elapsed_percent = tiles_elapsed as f32 / tiles_expected as f32;

            let completed_percent = self.trip_completed_percent();
            // println!("elapsed: {tiles_elapsed}, expected: {} percent: {completed_percent}", tiles_expected);

            if completed_percent > 0. {
                1. - (elapsed_percent - completed_percent)
            } else {
                1.
            }
        } else {
            1.
        }
    }

    pub fn trip_completed_percent(&self) -> f32 {
        if let Some(path) = &self.grid_path {
            self.path_index.max(0) as f32 / (path.0.len() - 1).max(1) as f32
        } else {
            1.
        }
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn test_successors() {
        let grid = Grid::new_from_string(
            "___
             >>>
             ___",
        );
        let suc = grid.road_successors(&grid.pos(1, 1));
        assert_eq!(suc, vec![(grid.pos(2, 1), 1)]);
    }

    #[test]
    fn test_path() {
        let grid = Grid::new_from_string(">>1");

        let path = grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .unwrap();
        assert_eq!(path.0, vec![grid.pos(1, 0), grid.pos(2, 0)]);
        assert_eq!(path.1, 1);
    }

    #[test]
    fn test_path_fail() {
        let grid = Grid::new_from_string("<<1");

        assert!(grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .is_none());
    }

    #[test]
    fn test_path_house() {
        let grid = Grid::new_from_string(
            "__<<h_
             _h>>__",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &1)
                .unwrap(),
            ((2..5).map(|i| grid.pos(i, 1)).collect(), 2)
        );
    }

    #[test]
    fn test_path_building_fail() {
        // let grid = Grid::new_from_string("_h>>h_");

        // assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(3, 0)).is_none());
        // assert!(grid.find_path(&grid.pos(0, 0), &grid.pos(5, 0)).is_none());
    }

    #[test]
    fn test_path_dead_end() {
        let grid = Grid::new_from_string(
            "h_*<
             __>*",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &1)
                .unwrap(),
            (
                vec![
                    grid.pos(2, 1),
                    grid.pos(3, 1),
                    grid.pos(3, 0),
                    grid.pos(2, 0),
                    grid.pos(1, 0),
                ],
                4
            )
        );
    }

    #[test]
    fn test_path_station() {
        let grid = Grid::new_from_string(
            "11<<22
             11>>22",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &2)
                .unwrap(),
            ((2..5).map(|i| grid.pos(i, 1)).collect(), 2)
        );
    }

    #[test]
    fn test_reserved() {
        let mut grid = Grid::new_from_string(">>>1");

        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(2, 0);

        // let mut vehicle = Vehicle::new(0, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

        // assert_eq!(
        //     Vehicle::reserve(&mut grid, 12, end_pos, &mut vehicle.reserved),
        //     Ok(())
        // );

        // assert_eq!(
        //     Vehicle::reserve(&mut grid, 12, end_pos, &mut vehicle.reserved),
        //     Err(ReservationError::TileReserved)
        // );
    }
}
