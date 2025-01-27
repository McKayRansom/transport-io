use std::u64;

use building::BuildingType;
use pathfinding::prelude::{astar, bfs_reach};
use tile::{PlanReservation, Tick, Tile};
use vehicle::{VehiclePosition, SPEED_TICKS_PER_TILE};

use super::*;

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;

impl Grid {
    pub fn reserve(
        &mut self,
        pos: &Position,
        vehicle_id: Id,
        current: Tick,
        start: Tick,
        end: Tick,
    ) -> Result<PlanReservation, ReservationError> {
        self.get_tile_mut(pos)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(vehicle_id, *pos, current, start, end)
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
        if let Some(Tile::Road(road)) = self.get_tile(pos) {
            road.get_connections(pos)
                .iter()
                .filter_map(|dir| {
                    let new_pos = *pos + *dir;
                    self.get_tile(&new_pos).map(|tile| (new_pos, tile.cost()))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn reserve_check_yield(
        &mut self,
        pos: &Position,
        id: Id,
        // should_yield: YieldType,
        tick: Tick,
        start: Tick,
        end: Tick,
    ) -> Result<PlanReservation, ReservePathError> {
        match self.reserve(pos, id, tick, start, end) {
            Ok(reservation) => {
                // if let Some(yield_to_pos) =
                    // self.should_we_yield_when_entering(should_yield, pos, id)
                // {
                    // Err(ReservePathError::Blocking(yield_to_pos))
                // } else {
                    Ok(reservation)
                // }
            }
            Err(ReservationError::TileInvalid) => Err(ReservePathError::InvalidPath),
            Err(ReservationError::TileReserved) => Err(ReservePathError::Blocking(*pos)),
        }
    }

    pub fn iter_reachable<FN>(&self, pos: Position, func: FN)
    where
        FN: Fn(Position),
    {
        for pos in bfs_reach(pos, |pos| {
            self.road_successors(pos)
                .into_iter()
                .map(|(pos, _cost)| pos)
        }) {
            func(pos);
        }
    }
}

pub enum ReservePathError {
    InvalidPath,
    Blocking(Position),
}

#[derive(Serialize, Deserialize)]
pub struct VehiclePath {
    id: Id,
    pub grid_path: Path,
    pub reserved: Vec<PlanReservation>,
    path_index: usize,
    path_time_ticks: u32,
    elapsed_ticks: u32,
    pub destination: Id,

    // This is an optimization and doesn't need to be saved
    #[serde(skip_serializing, skip_deserializing)]
    blocking_tile: Option<Position>,
}

impl VehiclePath {
    pub fn new(
        id: Id,
        grid: &mut Grid,
        start: (Position, Direction),
        destination: Id,
        tick: Tick,
    ) -> Result<Self, ReservationError> {
        let reservation = grid
            .get_tile_mut(&start.0)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(id, start.0, tick, 0, u64::MAX)?;

        let mut vehicle_path = Self {
            id,
            grid_path: None,
            path_time_ticks: 0,
            path_index: 0,
            destination,
            blocking_tile: None,
            elapsed_ticks: 0,
            reserved: vec![reservation],
        };

        vehicle_path.find_path(grid, start);

        Ok(vehicle_path)
    }

    pub fn fixup(&mut self, _grid: &mut Grid) -> Result<(), ReservationError> {
        // Fix serialization
        // for reservation in &mut self.reserved {
        //     *reservation = grid.reserve(&reservation.pos, self.id)?
        // }
        Ok(())
    }

    fn reserve_path(
        &mut self,
        grid: &mut Grid,
        tick: Tick,
        speed: u64,
    ) -> Result<Vec<PlanReservation>, ReservePathError> {
        // let should_yield = grid
        //     .get_tile(&current_pos)
        //     .ok_or(ReservePathError::InvalidPath)?
        //     .should_yield();

        let mut reserved = Vec::<PlanReservation>::new();

        let mut start = tick;
        let mut end = tick + speed;

        for pos in &self
            .grid_path
            .as_ref()
            .ok_or(ReservePathError::InvalidPath)?
            .0[self.path_index..]
        {
            // if let Some(pos) = self
            //     .grid_path
            //     .as_ref()
            //     .ok_or(ReservePathError::InvalidPath)?
            //     .0
            //     .get(self.path_index)
            // {
            reserved.push(grid.reserve_check_yield(pos, self.id, tick, start, end)?);

            if let Some(Tile::Road(road)) = grid.get_tile(pos) {
                if road.connection_count() > 1 {
                    start += speed;
                    end += speed;
                    continue;
                }
            }
            break;
        }

        Ok(reserved)
    }

    fn find_path(&mut self, grid: &mut Grid, start: (Position, Direction)) -> bool {
        self.grid_path = grid.find_path(start, &self.destination);

        if let Some(path) = &self.grid_path {
            self.path_time_ticks = path.1 * SPEED_TICKS_PER_TILE as u32;
            self.path_index = 0;
        }
        self.grid_path.is_some()
    }

    pub fn reserve_next_pos(
        &mut self,
        grid: &mut Grid,
        start: (Position, Direction),
        tick: Tick,
        speed: u64
    ) -> Option<Position> {
        match self.reserve_path(grid, tick, speed) {
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

    pub fn update_position(&mut self, path_grid: &mut Grid, pos: &mut VehiclePosition, tick: Tick, speed: u64) -> Status {
        // if let Some(blocking_tile) = self.blocking_tile {
        //     if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
        //         if road.reserved.is_reserved() {
        //             // don't bother
        //             return Status::EnRoute;
        //         }
        //     }
        // }
        // self.blocking_tile = None;

        if let Some(tile) = path_grid.get_tile(&pos.grid_pos) {
            if tile.get_building_id() == Some(self.destination) {
                return Status::ReachedDestination;
            }
        }

        let next_pos = self.reserve_next_pos(path_grid, (pos.grid_pos, pos.dir), tick, speed);

        pos.update_next_pos(next_pos);

        Status::EnRoute
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
            ">L>
             _._",
        );
        assert_eq!(
            grid.road_successors(&(0, 0).into()),
            vec![((1, 0).into(), 1)]
        );
        assert_eq!(
            grid.road_successors(&(1, 0).into()),
            vec![((2, 0).into(), 1), ((1, 1).into(), 1)]
        );
        assert_eq!(grid.road_successors(&(0, 1).into()), vec![]);
        assert_eq!(grid.road_successors(&(3, 3).into()), vec![]);
    }

    #[test]
    fn test_path() {
        let grid = Grid::new_from_string(">>1");

        let path = grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .unwrap();
        assert_eq!(path, (vec![grid.pos(1, 0), grid.pos(2, 0)], 1));
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
}

#[cfg(test)]
mod vehicle_path_tests {
    use super::*;

    fn reserve(grid: &mut Grid, pos: Position, tick: Tick, start: Tick, end: Tick) -> Result<PlanReservation, ReservationError> {
        grid.get_tile_mut(&pos).unwrap().reserve(1234, pos, tick, start, end)
    }

    #[test]
    fn intersection_traffic() {
        let mut grid = Grid::new_from_string(
            "LR>>1
             _^___",
        );

        let start: (Position, Direction) = ((1, 1).into(), Direction::UP);

        let mut path = VehiclePath::new(1, &mut grid, start, 1, 0).unwrap();
        assert!(reserve(&mut grid, start.0, 0, 0, 1).is_err());

        let _reservation = reserve(&mut grid, (1, 0).into(), 0, 0, 1).unwrap();

        assert_eq!(path.reserve_next_pos(&mut grid, start, 0, 4), None);

        assert_eq!(path.reserve_next_pos(&mut grid, start, 4, 4), Some((1, 0).into()));
    }

    #[test]
    fn yield_to_intersection_traffic() {
        // let mut grid = Grid::new_from_string(
        //     "LR>>1
        //      _y___",
        // );

        // let start: (Position, Direction) = ((1, 1).into(), Direction::UP);

        // let mut path = VehiclePath::new(1, &mut grid, start, 1).unwrap();
        // assert!(reserve(&mut grid, start.0).is_err());

        // let reservation = reserve(&mut grid, (0, 0).into()).unwrap();

        // assert!(grid
        //     .reserve_check_yield(&(1, 0).into(), path.id, YieldType::Never)
        //     .is_ok());
        // assert!(grid
        //     .reserve_check_yield(&(1, 0).into(), path.id, YieldType::IfAtIntersection)
        //     .is_err());

        // assert_eq!(path.reserve_next_pos(&mut grid, start), None);
        // assert_eq!(path.blocking_tile, Some(reservation.pos));

        // drop(reservation);

        // dbg!(&path.reserved);

        // if let Tile::Road(road) = grid.get_tile(&(0, 0).into()).unwrap() {
        //     dbg!(&road.reserved);
        // } else {
        //     panic!()
        // }

        // assert_eq!(
        //     grid.should_we_yield_when_entering(
        //         YieldType::IfAtIntersection,
        //         &(1, 0).into(),
        //         path.id
        //     ),
        //     None
        // );
        // assert!(grid
        //     .reserve_check_yield(&(1, 0).into(), path.id, YieldType::IfAtIntersection)
        //     .is_ok());

        // assert_eq!(path.reserve_next_pos(&mut grid, start), Some((1, 0).into()));
    }

    #[test]
    fn yield_roundabout_traffic() {
        // let mut grid = Grid::new_from_string(
        //     "\
        //     __.3__
        //     __.^__
        //     4<lr<<
        //     >>LR>2
        //     __.^__
        //     __1^__
        //     ",
        // );

        // // Top Vehicle is going straight down
        // let start_top: (Position, Direction) = ((2, 1).into(), Direction::DOWN);
        // let mut path_top = VehiclePath::new(1, &mut grid, start_top, 1).unwrap();

        // let start_left: (Position, Direction) = ((1, 3).into(), Direction::RIGHT);
        // let mut path_left = VehiclePath::new(2, &mut grid, start_left, 2).unwrap();

        // let start_bottom: (Position, Direction) = ((3, 4).into(), Direction::UP);
        // let mut path_bottom = VehiclePath::new(3, &mut grid, start_bottom, 3).unwrap();

        // let start_right: (Position, Direction) = ((4, 2).into(), Direction::LEFT);
        // let mut path_right = VehiclePath::new(4, &mut grid, start_right, 4).unwrap();

        // println!("grid: \n{:?}", grid);

        // assert!(path_top.reserve_next_pos(&mut grid, start_top).is_some());
        // assert!(path_left.reserve_next_pos(&mut grid, start_left).is_none());
        // assert!(path_bottom
        //     .reserve_next_pos(&mut grid, start_bottom)
        //     .is_some());
        // assert!(path_right
        //     .reserve_next_pos(&mut grid, start_bottom)
        //     .is_none());

        // println!("grid after: \n{:?}", grid);

        // assert!(path_top.blocking_tile.is_none());
        // assert!(path_left.blocking_tile.is_some());
        // assert!(path_bottom.blocking_tile.is_none());
        // assert!(path_right.blocking_tile.is_some());
    }

    #[test]
    fn yield_house_to_relevant_traffic() {
        // Houses should yield, but only to relevant traffic
        // let mut grid = Grid::new_from_string(
        //     "\
        //     <<<<
        //     >>>1
        //     _h__",
        // );

        // let start: (Position, Direction) = ((1, 2).into(), Direction::UP);

        // let mut path = VehiclePath::new(1, &mut grid, start, 1).unwrap();

        // let yield_to_pos = Position::new(0, 1);

        // let reservation = reserve(&mut grid, yield_to_pos).unwrap();

        // assert_eq!(path.reserve_next_pos(&mut grid, start), None);
        // assert_eq!(path.blocking_tile, Some(reservation.pos));

        // drop(reservation);

        // // reserve position accross the street
        // let do_not_yield_to_pos = Position::new(1, 0);
        // let reservation = reserve(&mut grid, do_not_yield_to_pos).unwrap();

        // assert_eq!(path.reserve_next_pos(&mut grid, start), Some((1, 1).into()));

        // drop(reservation);
    }
}
