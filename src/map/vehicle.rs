use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{
    grid::{Grid, ReservationError},
    path::VehiclePath,
    position::GRID_CELL_SIZE,
    Direction, Position,
};

const ACCEL_PIXELS_PER_TICK: u32 = 1;
const SPEED_PIXELS_PER_TICK: u32 = 16;
pub const SPEED_TICKS_PER_TILE: i16 = (GRID_CELL_SIZE.0 as i16 / (SPEED_PIXELS_PER_TICK / 4) as i16) * 4;
const HOPELESSLY_LATE_PERCENT: f32 = 0.5;

#[derive(Serialize, Deserialize)]
pub struct VehiclePosition {
    pub grid_pos: Position,
    pub dir: Direction,
    pub lag_pos: u32,
    pub lag_speed: u32,
}

impl VehiclePosition {
    pub fn new(start: (Position, Direction)) -> Self {
        Self {
            grid_pos: start.0,
            lag_pos: 0,
            lag_speed: 0,
            dir: start.1,
        }
    }

    fn update_speed(&mut self) {
        if self.lag_pos > 0 {
            self.lag_pos -= (self.lag_speed / 4).min(self.lag_pos);
        }
        if self.lag_speed < SPEED_PIXELS_PER_TICK {
            self.lag_speed += ACCEL_PIXELS_PER_TICK;
        }
    }

    pub fn update_next_pos(&mut self, pos: Option<Position>) {
        if let Some(next_pos) = pos {
            self.dir = next_pos - self.grid_pos;
            // self.dir.z = -self.dir.z;
            self.lag_pos = GRID_CELL_SIZE.0 as u32;
            self.update_speed();
            self.grid_pos = next_pos;
        } else {
            self.lag_speed = 0;
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    pub path: VehiclePath,
    pub pos: VehiclePosition,

    // this could be calculated from destination
    pub color: SpawnerColors,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd)]
pub enum Status {
    EnRoute,
    ReachedDestination,
    HopelesslyLate,
}

impl Vehicle {
    pub fn new(
        id: Id,
        start: (Position, Direction),
        destination: Id,
        grid: &mut Grid,
    ) -> Result<Self, ReservationError> {
        Ok(Self {
            path: VehiclePath::new(id, grid, start, destination)?,
            pos: VehiclePosition::new(start),
            color: SpawnerColors::Blue,
        })
    }

    pub fn fixup(&mut self, grid: &mut Grid) -> Result<(), ReservationError> {
        self.path.fixup(grid)
    }

    pub fn update(&mut self, path_grid: &mut Grid) -> Status {
        self.path.update_trip();
        if self.path.trip_late() < HOPELESSLY_LATE_PERCENT {
            Status::HopelesslyLate
        } else
        if self.pos.lag_pos != 0 {
            self.pos.update_speed();
            Status::EnRoute
        } else {
            self.path.update_position(path_grid, &mut self.pos)
        }
    }
}

#[cfg(test)]
mod vehicle_tests {

    use super::*;

    #[test]
    fn test_init() {
        let mut grid = Grid::new_from_string(">>>1");
        let start_pos = grid.pos(0, 0);
        let vehicle = Vehicle::new(1, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

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
    #[ignore = "unga bunga"]
    fn test_update_speed() {
        // let mut grid = Grid::new_from_string(">>>1");

        // let start_pos = grid.pos(0, 0);

        // let mut vehicle = Vehicle::new(0, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

        // vehicle.lag_pos = SPEED_PIXELS_PER_TICK;

        // vehicle.update_speed();

        // assert_eq!(vehicle.lag_pos, 0);
    }

    #[test]
    fn test_blocking_tile() {}
}
