use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{
    grid::{Grid, ReservationError},
    path::{Path, ReservePathError},
    position::GRID_CELL_SIZE,
    tile::{PlanReservation, Tick, Tile},
    Direction, Position,
};

const SPEED_PIXELS: u32 = 4;
pub const SPEED_TICKS: Tick = GRID_CELL_SIZE.0 as u64 / SPEED_PIXELS as u64;
const HOPELESSLY_LATE_PERCENT: f32 = 0.5;

#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    pub pos: Position,
    pub dir: Direction,
    // pub lag_pos: u32,

    // this could be calculated from destination
    pub color: SpawnerColors,

    id: Id,
    pub grid_path: Path,
    pub reserved: VecDeque<PlanReservation>,
    path_index: usize,
    path_time_ticks: u32,
    elapsed_ticks: u32,
    pub destination: Id,

    // This is an optimization and doesn't need to be saved
    #[serde(skip_serializing, skip_deserializing)]
    blocking_tile: Option<Position>,
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
        tick: Tick,
    ) -> Result<Self, ReservationError> {
        let reservation = grid
            .get_tile_mut(&start.0)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(id, start.0, tick, tick, u64::MAX)?;

        let mut vehicle = Self {
            pos: start.0,
            dir: start.1,
            // lag_pos: 0,
            color: SpawnerColors::Blue,

            id,
            grid_path: None,
            path_time_ticks: 0,
            path_index: 0,
            destination,
            blocking_tile: None,
            elapsed_ticks: 0,
            reserved: VecDeque::from([reservation]),
        };

        vehicle.find_path(grid);

        Ok(vehicle)
    }

    fn update_speed(&mut self) {
        // if self.lag_pos > 0 {
        // self.lag_pos -= (SPEED_PIXELS).min(self.lag_pos);
        // }
        // if self.lag_speed < SPEED_PIXELS_PER_TICK {
        //     self.lag_speed += ACCEL_PIXELS_PER_TICK;
        // }
    }

    pub fn update_next_pos(&mut self, pos: Option<Position>) {
        if let Some(next_pos) = pos {
            self.dir = next_pos - self.pos;
            // self.dir.z = -self.dir.z;
            // self.lag_pos = GRID_CELL_SIZE.0 as u32;
            self.update_speed();
            self.pos = next_pos;
        } //  else {
          // self.lag_speed = 0;
          // }
    }

    pub fn lag_pos(&self, tick: Tick) -> u32 {
        if let Some(reserve) = self.reserved.back() {
            if reserve.end == u64::MAX {
                0
            } else if reserve.end > tick {
                (reserve.end - tick).min(SPEED_TICKS) as u32 * SPEED_PIXELS
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn update(&mut self, path_grid: &mut Grid, tick: Tick) -> Status {
        self.update_trip();
        if self.trip_late() < HOPELESSLY_LATE_PERCENT {
            Status::HopelesslyLate
        } else if self.lag_pos(tick) != 0 {
            self.update_speed();
            Status::EnRoute
        } else {
            self.update_position(path_grid, tick)
        }
    }

    fn reserve_path(&mut self, grid: &mut Grid, tick: Tick) -> Result<(), ReservePathError> {
        // let should_yield = grid
        //     .get_tile(&current_pos)
        //     .ok_or(ReservePathError::InvalidPath)?
        //     .should_yield();

        // let mut end_res = self.reserved.pop_front().unwrap();

        let mut to_reserve: Vec<PlanReservation> = Vec::new();

        let mut start = tick;

        if let Some(head) = self.reserved.front() {
            // head.end could be u64::MAX
            start = if head.end == u64::MAX {
                head.start
            } else {
                head.end
            }
        }

        let mut end = start + SPEED_TICKS as u64;

        for pos in &self
            .grid_path
            .as_ref()
            .ok_or(ReservePathError::InvalidPath)?
            .0[self.path_index..]
        {
            match grid.is_reserved(pos, self.id, start, end) {
                Ok(()) => {
                    to_reserve.push(PlanReservation::new(*pos, start, end));
                }
                Err(ReservationError::TileInvalid) => return Err(ReservePathError::InvalidPath),
                Err(ReservationError::TileReserved) => {
                    return Err(ReservePathError::Blocking(*pos))
                }
            }

            if let Some(Tile::Road(road)) = grid.get_tile(pos) {
                if road.connection_count() > 1 {
                    start += SPEED_TICKS as u64;
                    end += SPEED_TICKS as u64;
                    continue;
                }
            }

            // we gotta check if we can for real stop here...
            match grid.is_reserved(pos, self.id, start, u64::MAX) {
                Ok(()) => {
                    *to_reserve.last_mut().unwrap() = PlanReservation::new(*pos, start, end);
                    break;
                }
                Err(ReservationError::TileInvalid) => return Err(ReservePathError::InvalidPath),
                Err(ReservationError::TileReserved) => {
                    return Err(ReservePathError::Blocking(*pos))
                }
            }
        }

        for res in to_reserve {
            self.path_index += 1;
            self.reserved.push_front(
                grid.reserve(&res.pos, self.id, tick, res.start, res.end)
                    .unwrap(),
            )
        }

        Ok(())
    }

    fn find_path(&mut self, grid: &mut Grid) -> bool {
        self.grid_path = grid.find_path((self.pos, self.dir), &self.destination);

        if let Some(path) = &self.grid_path {
            self.path_time_ticks = path.1 * SPEED_TICKS as u32;
            self.path_index = 0;
        }
        self.grid_path.is_some()
    }

    pub fn reserve_next_pos(&mut self, grid: &mut Grid, tick: Tick) -> Option<Position> {
        match self.reserve_path(grid, tick) {
            Ok(()) => {
                // self.path_index += 1;
                if let Some(res) = self.reserved.pop_back() {
                    // TODO: Consider reserving it for a few ticks
                    grid.get_tile_mut(&res.pos).unwrap().unreserve(self.id);
                    self.reserved.back().map(|res| res.pos)
                } else {
                    None
                }
            }
            Err(ReservePathError::InvalidPath) => {
                self.find_path(grid);
                None
            }
            Err(ReservePathError::Blocking(blocking_pos)) => {
                self.blocking_tile = Some(blocking_pos);
                // roll back reserved
                None
            }
        }
    }

    pub fn update_trip(&mut self) {
        self.elapsed_ticks += 1;
    }

    pub fn update_position(&mut self, path_grid: &mut Grid, tick: Tick) -> Status {
        // if let Some(blocking_tile) = self.blocking_tile {
        //     if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
        //         if road.reserved.is_reserved() {
        //             // don't bother
        //             return Status::EnRoute;
        //         }
        //     }
        // }
        // self.blocking_tile = None;

        if let Some(tile) = path_grid.get_tile(&self.pos) {
            if tile.get_building_id() == Some(self.destination) {
                return Status::ReachedDestination;
            }
        }

        let next_pos = self.reserve_next_pos(path_grid, tick);

        self.update_next_pos(next_pos);

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.grid_path {
            let tiles_elapsed = (self.elapsed_ticks.saturating_sub(1) / SPEED_TICKS as u32) + 1;
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
mod vehicle_tests {

    // use super::*;

    #[test]
    fn test_init() {
        // let mut grid = Grid::new_from_string(">>>1");
        // let start_pos = grid.pos(0, 0);
        // let vehicle = Vehicle::new(1, (start_pos, Direction::RIGHT), 1, &mut grid).unwrap();

        // assert!(Vehicle::new(2, (start_pos, Direction::RIGHT), 1, &mut grid).is_err());

        // drop(vehicle)
    }

    #[test]
    fn test_status() {
        // let mut grid = Grid::new_from_string(">>>>1");
        // let mut vehicle =
        //     Vehicle::new(0, (grid.pos(0, 0), Direction::RIGHT), 1, &mut grid).unwrap();

        // // let
        // // let reservation = grid.get_tile_mut(&grid.pos(1, 0)).reserve(1).unwrap();

        // assert_eq!(vehicle.update(&mut grid), Status::EnRoute);

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

#[cfg(test)]
mod vehicle_path_tests {
    use std::u64;

    use crate::map::tile::{PlanReserved, PlanReservedList};

    use super::*;

    fn reserve(
        grid: &mut Grid,
        pos: Position,
        tick: Tick,
        start: Tick,
        end: Tick,
    ) -> Result<PlanReservation, ReservationError> {
        grid.get_tile_mut(&pos)
            .unwrap()
            .reserve(1234, pos, tick, start, end)
    }

    fn unreserve(grid: &mut Grid, res: PlanReservation) {
        grid.get_tile_mut(&res.pos).unwrap().unreserve(1234);
    }

    fn get_reserved(grid: &mut Grid, pos: Position) -> Result<PlanReservedList, ReservationError> {
        if let Some(Tile::Road(road)) = grid.get_tile(&pos) {
            Ok(road.reserved.clone())
        } else {
            Err(ReservationError::TileInvalid)
        }
    }

    #[test]
    fn intersection_traffic() {
        let mut grid = Grid::new_from_string(
            "LR>>1
             _^___",
        );

        let start: (Position, Direction) = ((1, 1).into(), Direction::UP);

        let mut vehicle = Vehicle::new(1, start, 1, &mut grid, 0).unwrap();

        assert_eq!(vehicle.pos, start.0);
        assert!(reserve(&mut grid, start.0, 0, 0, 1).is_err());
        assert_eq!(vehicle.path_index, 0);
        assert_eq!(vehicle.lag_pos(0), 0);
        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new(start.0, 0, u64::MAX)
        );
        assert_eq!(
            get_reserved(&mut grid, start.0).unwrap(),
            ([PlanReserved::new(vehicle.id, 0, u64::MAX)].as_slice()).into()
        );

        // block the intersection exit
        let reservation = reserve(&mut grid, (2, 0).into(), 0, 0, u64::MAX).unwrap();
        vehicle.update(&mut grid, 0);
        assert_eq!(vehicle.path_index, 0);
        assert_eq!(vehicle.pos, start.0);

        // assert_eq!(vehicle.reserve_next_pos(&mut grid, 0), None);
        // assert_eq!(vehicle.reserve_next_pos(&mut grid, 4), Some((1, 0).into()));

        unreserve(&mut grid, reservation);

        let mut tick = 0;

        vehicle.update(&mut grid, tick);

        assert_eq!(
            *vehicle.reserved.back().unwrap(),
            PlanReservation::new((1, 0).into(), tick, SPEED_TICKS)
        );
        assert_eq!(vehicle.pos, (1, 0).into());

        assert_eq!(
            *vehicle.reserved.front().unwrap(),
            PlanReservation::new((2, 0).into(), tick + SPEED_TICKS, SPEED_TICKS * 2)
        );

        assert_eq!(vehicle.path_index, 2);
        assert_eq!(vehicle.lag_pos(tick), 32);
        assert_eq!(
            get_reserved(&mut grid, (1, 0).into()).unwrap(),
            ([PlanReserved::new(1, 0, 8)].as_slice()).into()
        );
        assert_eq!(
            get_reserved(&mut grid, (2, 0).into()).unwrap(),
            ([PlanReserved::new(1, 8, 16)].as_slice()).into()
        );

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        // assert_eq!(vehicle.lag_pos(tick), 0);
        assert_eq!(vehicle.pos, (2, 0).into());
        assert_eq!(vehicle.path_index, 3);

        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new((3, 0).into(), 16, 24)
        );

        assert_eq!(
            get_reserved(&mut grid, (3, 0).into()).unwrap(),
            ([PlanReserved::new(1, 16, 24)].as_slice()).into()
        );

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        assert_eq!(vehicle.pos, (3, 0).into());
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
