use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{
    grid::{Grid, ReservationError}, path::Path, position::GRID_CELL_SIZE, tile::{Reservation, Tile}, Direction, Position
};

const ACCEL_PIXELS_PER_TICK: u32 = 1;
const SPEED_PIXELS_PER_TICK: u32 = 4;
const SPEED_TICKS_PER_TILE: i16 = GRID_CELL_SIZE.0 as i16 / SPEED_PIXELS_PER_TICK as i16;
const HOPELESSLY_LATE_PERCENT: f32 = 0.5;

#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Id,
    // pathing
    pub path: Path,
    reserved: Vec<Reservation>,
    path_index: usize,
    path_time_ticks: u32,
    elapsed_ticks: u32,
    pub destination: Id,
    // position
    pub pos: Position,
    pub lag_pos: u32,
    pub lag_speed: u32,
    pub dir: Direction,
    // This is an optimization and doesn't need to be saved
    #[serde(skip_serializing, skip_deserializing)]
    blocking_tile: Option<Position>,

    // this could be calculated from destination
    pub color: SpawnerColors,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd)]
pub enum Status {
    EnRoute,
    ReachedDestination,
    HopelesslyLate,
}

pub enum ReservePathError {
    InvalidPath,
    Blocking(Position),
}

impl Vehicle {
    pub fn new(
        id: Id,
        start: (Position, Direction),
        destination: Id,
        grid: &mut Grid,
    ) -> Result<Self, ReservationError> {
        let reservation = grid
            .get_tile_mut(&start.0)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(id, start.0)?;

        let mut vehicle = Vehicle {
            id,
            path: None,
            path_time_ticks: 0,
            path_index: 0,
            destination,
            elapsed_ticks: 0,
            pos: start.0,
            lag_pos: 0,
            lag_speed: 0,
            dir: start.1,
            blocking_tile: None,
            color: SpawnerColors::Blue,
            reserved: vec![reservation],
        };

        vehicle.find_path(grid);

        Ok(vehicle)
    }

    pub fn fixup(&mut self, grid: &mut Grid) -> Result<(), ReservationError> {
        // Fix serialization
        for reservation in &mut self.reserved {
            *reservation = grid.reserve(&reservation.pos, self.id)?
        }

        Ok(())
    }

    fn reserve(
        path_grid: &mut Grid,
        vehicle_id: Id,
        position: Position,
        reserved: &mut Vec<Reservation>,
    ) -> Result<(), ReservationError> {
        let reservation = path_grid.reserve(&position, vehicle_id)?;
        reserved.push(reservation);
        Ok(())
    }

    fn reserve_path(&self, grid: &mut Grid) -> Result<Vec<Reservation>, ReservePathError> {
        // TODO: Move to grid

        let should_yield = grid
            .get_tile(&self.pos)
            .ok_or(ReservePathError::InvalidPath)?
            .should_yield();

        let (path, _cost) = self.path.as_ref().ok_or(ReservePathError::InvalidPath)?;

        let mut reserved = Vec::<Reservation>::new();

        // for pos in &path[self.path_index + 1..] {
        if let Some(pos) = path.get(self.path_index) {
            // TODO Make function
            match Vehicle::reserve(grid, self.id, *pos, &mut reserved) {
                Ok(_) => {
                    if let Some(yield_to_pos) =
                        grid.should_we_yield_when_entering(should_yield, pos)
                    {
                        return Err(ReservePathError::Blocking(yield_to_pos));
                    }
                    // Fall through
                }
                Err(ReservationError::TileInvalid) => {
                    return Err(ReservePathError::InvalidPath);
                }
                Err(ReservationError::TileReserved) => {
                    return Err(ReservePathError::Blocking(*pos));
                }
            }
        }

        Ok(reserved)
    }

    fn update_speed(&mut self) {
        if self.lag_pos > 0 {
            self.lag_pos -= self.lag_speed.min(self.lag_pos);
        }
        if self.lag_speed < SPEED_PIXELS_PER_TICK {
            self.lag_speed += ACCEL_PIXELS_PER_TICK;
        }
    }

    fn find_path(&mut self, grid: &mut Grid) -> bool {
        self.path = grid.find_path((self.pos, self.dir), &self.destination);

        if let Some(path) = &self.path {
            self.path_time_ticks = path.1 * SPEED_TICKS_PER_TILE as u32;
            self.path_index = 0;
        }
        self.path.is_some()
    }

    fn get_next_pos(&mut self, grid: &mut Grid) -> Option<Position> {
        match self.reserve_path(grid) {
            Ok(reserved) => {
                self.reserved = reserved;
                if let Some(path) = self.path.as_ref() {
                    let pos = path.0[self.path_index];
                    self.path_index += 1;
                    Some(pos)
                } else {
                    None
                }
            }
            Err(ReservePathError::InvalidPath) => {
                self.find_path(grid);
                // we're pretty well screwed if this happens so maybe don't do this??
                // TODO: Don't do this, just unreserve when we find a path!
                let _ = Vehicle::reserve(grid, self.id, self.pos, &mut self.reserved);
                None
            }
            Err(ReservePathError::Blocking(blocking_pos)) => {
                self.blocking_tile = Some(blocking_pos);
                let _ = Vehicle::reserve(grid, self.id, self.pos, &mut self.reserved);
                // grid.reserve_position(&self.pos, self.id);
                None
            }
        }
    }

    fn update_position(&mut self, path_grid: &mut Grid) -> Status {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
                if road.reserved.is_reserved() {
                    // don't bother
                    return Status::EnRoute;
                }
            }
        }
        self.blocking_tile = None;

        if let Some(tile) = path_grid.get_tile(&self.pos) {
            if tile.get_building_id() == Some(self.destination) {
                return Status::ReachedDestination;
            }
        }

        self.reserved.clear();

        if let Some(next_pos) = self.get_next_pos(path_grid) {
            self.dir = next_pos - self.pos;
            // self.dir.z = -self.dir.z;
            self.lag_pos = GRID_CELL_SIZE.0 as u32;
            self.update_speed();
            self.pos = next_pos;
        } else {
            self.lag_speed = 0;
        }

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.path {
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
        if let Some(path) = &self.path {
            self.path_index.max(0) as f32 / (path.0.len() - 1).max(1) as f32
        } else {
            1.
        }
    }

    pub fn update(&mut self, path_grid: &mut Grid) -> Status {
        self.elapsed_ticks += 1;
        if self.trip_late() < HOPELESSLY_LATE_PERCENT {
            Status::HopelesslyLate
        } else if self.lag_pos != 0 {
            self.update_speed();
            Status::EnRoute
        } else {
            self.update_position(path_grid)
        }
    }
}

#[cfg(test)]
mod vehicle_tests {

    use super::*;

    fn reserve(grid: &mut Grid, pos: Position) -> Result<Reservation, ReservationError> {
        grid.get_tile_mut(&pos).unwrap().reserve(1234, pos)
    }

    #[test]
    fn test_init() {
        let mut grid = Grid::new_from_string(">>>1");
        let start_pos = grid.pos(0, 0);
        let vehicle = Vehicle::new(1, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

        assert_eq!(
            reserve(&mut grid, start_pos).unwrap_err(),
            ReservationError::TileReserved
        );

        assert!(Vehicle::new(2, (start_pos, Direction::RIGHT), 1, &mut grid).is_err());

        drop(vehicle)
    }

    #[test]
    fn test_status() {
        let mut grid = Grid::new_from_string(">>>>1");
        let mut vehicle =
            Vehicle::new(0, (grid.pos(0, 0), Direction::RIGHT), 1, &mut grid).unwrap();

        // let
        // let reservation = grid.get_tile_mut(&grid.pos(1, 0)).reserve(1).unwrap();

        assert_eq!(vehicle.update(&mut grid), Status::EnRoute);

        // drop(reservation);
    }

    #[test]
    fn test_late() {
        // let mut grid = Grid::new_from_string(">>>>");
        // let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, grid.pos(3, 0), &mut grid).unwrap();

        // vehicle.update(&mut grid);

        // vehicle.elapsed_ticks = SPEED_TICKS_PER_TILE as u32 + 1;
        // assert_eq!(vehicle.trip_late(), 0.6666666);

        // vehicle.elapsed_ticks = (SPEED_TICKS_PER_TILE * 2) as u32 + 1;
        // assert_eq!(vehicle.trip_late(), 0.33333337);
    }

    #[test]
    #[ignore = "why is this so complicated??? Do we even need this?"]
    fn test_trip() {
        // let mut grid = Grid::new_from_string(">>>>");
        // let destination = grid.pos(3, 0);
        // let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, destination, &mut grid).unwrap();

        // let trip_length: u32 = 3;
        // let trip_time = SPEED_TICKS_PER_TILE as u32 * trip_length;

        // assert_eq!(vehicle.path_time_ticks, trip_time);

        // assert_eq!(vehicle.trip_completed_percent(), 0.);

        // assert_eq!(vehicle.trip_late(), 1.0);
        // // grid.reserve_position(&(1, 0).into(), 1);

        // // assert_eq!(vehicle.update(&mut grid), Status::EnRoute);

        // for i in 0..(trip_length * SPEED_TICKS_PER_TILE as u32) {
        //     assert_eq!(
        //         vehicle.update(&mut grid),
        //         Status::EnRoute,
        //         "Failed on tick {i}"
        //     );
        //     assert_eq!(
        //         vehicle.path_index,
        //         1 + (i / (SPEED_TICKS_PER_TILE as u32)) as usize,
        //         "Failed on tick {i}"
        //     );
        //     assert_eq!(vehicle.elapsed_ticks, i + 1);
        //     assert_eq!(
        //         vehicle.trip_completed_percent(),
        //         ((i + SPEED_TICKS_PER_TILE as u32) / SPEED_TICKS_PER_TILE as u32) as f32
        //             / trip_length as f32,
        //         "Failed on tick {i}"
        //     );
        //     assert_eq!(
        //         vehicle.trip_late(),
        //         1.0,
        //         "Failed on tick {i} %{}",
        //         vehicle.trip_completed_percent()
        //     );
        // }

        // println!("Vehicle : {:?}", vehicle.blocking_tile);
        // assert_eq!(vehicle.update(&mut grid), Status::ReachedDestination);
        // assert_eq!(vehicle.pos, destination);
        // assert_ne!(vehicle.trip_late(), 1.0);
    }

    #[test]
    fn test_reserved() {
        let mut grid = Grid::new_from_string(">>>1");

        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(2, 0);

        let mut vehicle = Vehicle::new(0, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

        assert_eq!(
            Vehicle::reserve(&mut grid, 12, end_pos, &mut vehicle.reserved),
            Ok(())
        );

        assert_eq!(
            Vehicle::reserve(&mut grid, 12, end_pos, &mut vehicle.reserved),
            Err(ReservationError::TileReserved)
        );
    }

    #[test]
    #[ignore = "unga bunga"]
    fn test_update_speed() {
        let mut grid = Grid::new_from_string(">>>1");

        let start_pos = grid.pos(0, 0);

        let mut vehicle = Vehicle::new(0, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

        vehicle.lag_pos = SPEED_PIXELS_PER_TICK;

        vehicle.update_speed();

        assert_eq!(vehicle.lag_pos, 0);
    }

    #[test]
    fn test_blocking_tile() {}

    #[test]
    #[ignore = "f"]
    fn test_yield() {
        let mut grid = Grid::new_from_string(
            "\
            >>>>1
            _h___",
        );

        // println!("grid: {:?}", &grid);

        let start_pos = grid.pos(1, 1);
        let yield_to_pos = grid.pos(0, 0);

        let mut vehicle = Vehicle::new(0, (start_pos, Direction::UP), 1, &mut grid).unwrap();

        let reservation = reserve(&mut grid, yield_to_pos).unwrap();

        vehicle.update(&mut grid);

        // vehicle should not move
        assert_eq!(vehicle.pos, start_pos);
        assert_eq!(vehicle.blocking_tile.unwrap(), yield_to_pos);

        drop(reservation)
    }

    #[test]
    fn test_yield_roundabout() {
        // let mut grid = Grid::new_from_string(
        //     "\
        //     __.^__
        //     __.^__
        //     <<lr<<
        //     >>LR>>
        //     __.^__
        //     __.^__
        //     ",
        // );

        // let mut vehicle_top = Vehicle::new(0, grid.pos(2, 1), 0, grid.pos(2, 4), &mut grid).unwrap();

        // let mut vehicle_left = Vehicle::new(1, grid.pos(1, 3), 1, grid.pos(5, 3), &mut grid).unwrap();

        // let mut vehicle_bottom =
        //     Vehicle::new(2, grid.pos(3, 4), 2, grid.pos(3, 0), &mut grid).unwrap();

        // let mut vehicle_right = Vehicle::new(3, grid.pos(4, 2), 3, grid.pos(0, 2), &mut grid).unwrap();

        // assert!(vehicle_top.path.is_some());
        // assert!(vehicle_left.path.is_some());
        // assert!(vehicle_bottom.path.is_some());
        // assert!(vehicle_right.path.is_some());

        // println!("grid: \n{:?}", grid);

        // vehicle_top.update(&mut grid);
        // vehicle_left.update(&mut grid);
        // vehicle_bottom.update(&mut grid);
        // vehicle_right.update(&mut grid);

        // println!("grid after: \n{:?}", grid);

        // assert!(vehicle_top.blocking_tile.is_none());
        // assert!(vehicle_left.blocking_tile.is_some());
        // assert!(vehicle_bottom.blocking_tile.is_none());
        // assert!(vehicle_right.blocking_tile.is_some());
    }

    #[test]
    fn test_yield_building() {
        // Houses should yield, but only to relevant traffic
        // let mut grid = Grid::new_from_string(
        //     "\
        //     <<<<
        //     >>>>
        //     _h__",
        // );

        // let mut vehicle = Vehicle::new(grid.pos(1, 2), 0, grid.pos(3, 1), &mut grid).unwrap();

        // let yield_to_pos = grid.pos(0, 1);

        // assert!(vehicle.path.is_some());

        // // reserve position we should yield to
        // let reservation = reserve(&mut grid, yield_to_pos).unwrap();

        // vehicle.update(&mut grid);

        // assert_eq!(vehicle.blocking_tile.unwrap(), yield_to_pos);

        // // grid.unreserve_position(&yield_to_pos);
        // drop(reservation);

        // // reserve position accross the street
        // let do_not_yield_to_pos = grid.pos(1, 0);
        // let reservation = reserve(&mut grid, do_not_yield_to_pos).unwrap();

        // vehicle.update(&mut grid);

        // assert_eq!(vehicle.blocking_tile, None);

        // drop(reservation);
    }
}
