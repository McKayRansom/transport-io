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

const SPEED_PIXELS: i32 = 4;
pub const SPEED_TICKS: Tick = GRID_CELL_SIZE.0 as u64 / SPEED_PIXELS as u64;
const HOPELESSLY_LATE_PERCENT: f32 = 0.5;

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn cleanup(&self, grid: &mut Grid) {
        for res in &self.reserved {
            if let Some(tile) = grid.get_tile_mut(&res.pos) {
                tile.unreserve(self.id);
            }
        }
    }

    pub fn lead_pos(&self, tick: Tick) -> i32 {

        // if we are stuck
        if self.reserved.len() <= 1 {
            return 0;
        }

        if let Some(reserve) = self.reserved.back() {
            // reserve.start being less than tick is a bug
            // assert!(reserve.start > tick);
            if reserve.end == u64::MAX {
                0
            } else {
                (tick - (reserve.end - SPEED_TICKS)) as i32 * SPEED_PIXELS
            }
        } else {
            0
        }
    }

    pub fn update(&mut self, grid: &mut Grid, tick: Tick) -> Status {
        self.update_trip();
        if self.trip_late() < HOPELESSLY_LATE_PERCENT {
            return Status::HopelesslyLate;
        } // else if self.lead_pos(tick) < (SPEED_TICKS as u32 * 2) || self.lead_pos(tick) == 0 {
          // self.update_speed();
          // Status::EnRoute
          // } else {
        if let Some(res) = self.reserved.back() {
            // dbg!(res);
            if res.end > tick && res.end != u64::MAX {
                return Status::EnRoute;
            }

            if res.end <= tick {
                self.reserved.pop_back();
                self.update_next_pos();
            }
        }
        self.update_position(grid, tick)
        // }
        // return Status::EnRoute;
    }


    fn update_speed(&mut self) {
        // if self.lag_pos > 0 {
        // self.lag_pos -= (SPEED_PIXELS).min(self.lag_pos);
        // }
        // if self.lag_speed < SPEED_PIXELS_PER_TICK {
        //     self.lag_speed += ACCEL_PIXELS_PER_TICK;
        // }
    }

    fn update_next_pos(&mut self) {
        let mut iter = self.reserved.iter().rev();
        if let Some(res) = iter.next() {
            self.dir = res.pos - self.pos;
            if let Some(res_next) = iter.next() {
                self.dir = res_next.pos - res.pos;
            }
            // if let Some(res_after) = self.reserved.ba
            // self.dir.z = -self.dir.z;
            // self.lag_pos = GRID_CELL_SIZE.0 as u32;
            // self.update_speed();
            self.pos = res.pos;
        } //  else {
          // self.lag_speed = 0;
          // }
    }


    fn reserve_path(&mut self, grid: &mut Grid, tick: Tick) -> Result<(), ReservePathError> {
        let mut to_reserve: Vec<PlanReservation> = Vec::new();

        let mut start = if let Some(head) = self.reserved.front() {
            if head.end == u64::MAX {
                // this is not correct sadly :(
                // if we had to wait, this could be wrong
                head.start.max(tick) + SPEED_TICKS 
            } else {
                head.end
            }
        } else {
            tick
        };

        let mut end = start + SPEED_TICKS as u64;

        for pos in &self
            .grid_path
            .as_ref()
            .ok_or(ReservePathError::InvalidPath)?
            .0[self.path_index..]
        {
            // dbg!(pos, start, end);
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
                    *to_reserve.last_mut().unwrap() = PlanReservation::new(*pos, start, u64::MAX);
                    break;
                }
                Err(ReservationError::TileInvalid) => return Err(ReservePathError::InvalidPath),
                Err(ReservationError::TileReserved) => {
                    return Err(ReservePathError::Blocking(*pos))
                }
            }
        }

        if to_reserve.len() > 0 {
            let res = self.reserved.front().unwrap();
            if res.end == u64::MAX {
                // fixup this reservation
                let new_res = grid.get_tile_mut(&res.pos).unwrap().reserve(
                    self.id,
                    res.pos,
                    tick,
                    res.start,
                    to_reserve[0].start,
                );
                *self.reserved.front_mut().unwrap() = new_res.unwrap();
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
            self.path_time_ticks = (path.1 + 1) * SPEED_TICKS as u32;
            self.path_index = 0;
        }
        self.grid_path.is_some()
    }

    pub fn reserve_next_pos(&mut self, grid: &mut Grid, tick: Tick)  {
        match self.reserve_path(grid, tick) {
            Ok(()) => {
                // self.path_index += 1;

            }
            Err(ReservePathError::InvalidPath) => {
                self.find_path(grid);
            }
            Err(ReservePathError::Blocking(blocking_pos)) => {
                self.blocking_tile = Some(blocking_pos);
                // roll back reserved
            }
        }
    }

    pub fn update_trip(&mut self) {
        self.elapsed_ticks += 1;
    }

    pub fn update_position(&mut self, grid: &mut Grid, tick: Tick) -> Status {
        // if let Some(blocking_tile) = self.blocking_tile {
        //     if let Some(Tile::Road(road)) = grid.get_tile(&blocking_tile) {
        //         if road.reserved.is_reserved() {
        //             // don't bother
        //             return Status::EnRoute;
        //         }
        //     }
        // }
        // self.blocking_tile = None;

        if let Some(tile) = grid.get_tile(&self.pos) {
            if tile.get_building_id() == Some(self.destination) {
                return Status::ReachedDestination;
            }
        }

        self.reserve_next_pos(grid, tick);

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.grid_path {
            let tiles_elapsed = (self.elapsed_ticks.saturating_sub(1) / SPEED_TICKS as u32) + 1;
            let tiles_expected = path.1 + 1;

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
            self.path_index.max(0) as f32 / (path.0.len()/*  - 1*/).max(1) as f32
        } else {
            1.
        }
    }
}

#[cfg(test)]
mod vehicle_tests {

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
    fn test_init() {
        let mut grid = Grid::new_from_string(">>>1");
        let start_pos = grid.pos(0, 0);
        let _vehicle = Vehicle::new(1, (start_pos, Direction::RIGHT), 1, &mut grid, 0).unwrap();

        assert!(Vehicle::new(2, (start_pos, Direction::RIGHT), 1, &mut grid, 0).is_err());
    }

    #[test]
    fn test_lead_pos() {
        let mut grid = Grid::new_from_string(">>>>1");
        let tick: Tick = 0;
        let mut vehicle =
            Vehicle::new(1, ((0, 0).into(), Direction::RIGHT), 1, &mut grid, 0).unwrap();

        assert_eq!(vehicle.lead_pos(tick), 0);

        for tick in 0..16 {
            assert_eq!(vehicle.update(&mut grid, tick), Status::EnRoute);
            assert_eq!(vehicle.path_index as u64, (tick) / SPEED_TICKS + 1, "i: {tick}");
            assert_eq!(vehicle.pos, (0 + tick as i16 / SPEED_TICKS as i16 , 0).into(), "i: {tick}");
            assert_eq!(vehicle.pos, vehicle.reserved.back().unwrap().pos, "i: {tick}");
            assert_eq!(vehicle.lead_pos(tick), (tick as i32 * SPEED_PIXELS) % GRID_CELL_SIZE.0 as i32, "i: {tick}");
            println!("{}: {:?} + {}", tick, vehicle.pos, vehicle.lead_pos(tick));
        }

        // assert!(false);
    }

    #[test]
    fn test_straight() {
        let mut grid = Grid::new_from_string(">>>>1");
        let tick: Tick = 0;
        let mut vehicle =
            Vehicle::new(42, ((0, 0).into(), Direction::RIGHT), 1, &mut grid, 0).unwrap();

        // starting
        assert_eq!(vehicle.pos, (0, 0).into());
        assert_eq!(
            vehicle.reserved.back().unwrap(),
            &PlanReservation::new((0, 0).into(), 0, u64::MAX)
        );

        vehicle.update(&mut grid, tick);

        assert_eq!(vehicle.pos, (0, 0).into());
        assert_eq!(vehicle.reserved.len(), 2);
        assert_eq!(
            vehicle.reserved.back().unwrap(),
            &PlanReservation::new((0, 0).into(), 0, 8)
        );
        assert_eq!(
            vehicle.reserved.front().unwrap(),
            &PlanReservation::new((1, 0).into(), 8, u64::MAX)
        );

        vehicle.update(&mut grid, 8);

        assert_eq!(vehicle.pos, (1, 0).into());
        assert_eq!(vehicle.reserved.len(), 2);
        assert_eq!(
            vehicle.reserved.back().unwrap(),
            &PlanReservation::new((1, 0).into(), 8, 16)
        );
        assert_eq!(
            vehicle.reserved.front().unwrap(),
            &PlanReservation::new((2, 0).into(), 16, u64::MAX)
        );

        vehicle.update(&mut grid, 16);

        assert_eq!(vehicle.pos, (2, 0).into());
        assert_eq!(vehicle.reserved.len(), 2);
        assert_eq!(
            vehicle.reserved.back().unwrap(),
            &PlanReservation::new((2, 0).into(), 16, 24)
        );
        assert_eq!(
            vehicle.reserved.front().unwrap(),
            &PlanReservation::new((3, 0).into(), 24, u64::MAX)
        );
    }

    #[test]
    fn test_late() {
        let mut grid = Grid::new_from_string(">>>1");
        let mut vehicle = Vehicle::new(1, (grid.pos(0, 0), Direction::RIGHT), 1, &mut grid, 0).unwrap();

        vehicle.update(&mut grid, 0);

        // vehicle.elapsed_ticks = SPEED_TICKS as u32 + 1;
        // assert_eq!(vehicle.trip_late(), 0.6666666);

        // vehicle.elapsed_ticks = (SPEED_TICKS * 2) as u32 + 1;
        // assert_eq!(vehicle.trip_late(), 0.33333337);
    }

    #[test]
    fn test_trip() {
        let mut grid = Grid::new_from_string(">>>1");
        let mut vehicle = Vehicle::new(1, (grid.pos(0, 0), Direction::RIGHT), 1, &mut grid, 0).unwrap();

        let trip_length: u32 = 3;
        let trip_time = SPEED_TICKS as u32 * trip_length;

        assert_eq!(vehicle.path_time_ticks, trip_time);

        assert_eq!(vehicle.trip_completed_percent(), 0.);

        assert_eq!(vehicle.trip_late(), 1.0);
        // let _res = reserve(&mut grid, (1, 0).into(), 0, 0, u64::MAX);

        // assert_eq!(vehicle.update(&mut grid, 0), Status::EnRoute);

        for i in 0..(trip_length * SPEED_TICKS as u32) {
            assert_eq!(
                vehicle.update(&mut grid, i as u64),
                Status::EnRoute,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.path_index,
                1 + (i / (SPEED_TICKS as u32)) as usize,
                "Failed on tick {i}"
            );
            assert_eq!(vehicle.elapsed_ticks, i + 1);
            assert_eq!(
                vehicle.trip_completed_percent(),
                ((i + SPEED_TICKS as u32) / SPEED_TICKS as u32) as f32
                    / trip_length as f32,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.trip_late(),
                1.0,
                "Failed on tick {i} %{}",
                vehicle.trip_completed_percent()
            );
        }

        // println!("Vehicle : {:?}", vehicle.blocking_tile);
        // assert_eq!(vehicle.update(&mut grid), Status::ReachedDestination);
        // assert_eq!(vehicle.pos, destination);
        // assert_ne!(vehicle.trip_late(), 1.0);
    }



    #[test]
    fn test_blocking_tile() {}

    #[test]
    fn instersection_do_not_block() {
        let mut grid = Grid::new_from_string(
            "LR>>1
             _^___",
        );

        let start: (Position, Direction) = ((1, 1).into(), Direction::UP);

        let mut vehicle = Vehicle::new(1, start, 1, &mut grid, 0).unwrap();

        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new(start.0, 0, u64::MAX)
        );
        assert_eq!(
            get_reserved(&mut grid, start.0).unwrap(),
            ([PlanReserved::new(vehicle.id, 0, u64::MAX)].as_slice()).into()
        );

        let reservation = reserve(&mut grid, (2, 0).into(), 0, 0, 16).unwrap();
        vehicle.update(&mut grid, 0);
        assert_eq!(vehicle.path_index, 0);
        assert_eq!(vehicle.pos, start.0);

        // assert_eq!(vehicle.reserve_next_pos(&mut grid, 0), None);
        // assert_eq!(vehicle.reserve_next_pos(&mut grid, 4), Some((1, 0).into()));

        unreserve(&mut grid, reservation);

        let mut tick = 0;

        vehicle.update(&mut grid, tick);

        assert_eq!(
            vehicle.reserved[2],
            PlanReservation::new((1, 1).into(), 0, 8)
        );
        assert_eq!(
            vehicle.reserved[1],
            PlanReservation::new((1, 0).into(), 8, 16)
        );
        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new((2, 0).into(), 16, u64::MAX)
        );

        assert_eq!(vehicle.path_index, 2);
        // assert_eq!(vehicle.lead_pos(tick), 32);
        // assert_eq!(
        //     get_reserved(&mut grid, (1, 0).into()).unwrap(),
        //     ([PlanReserved::new(1, 0, 8)].as_slice()).into()
        // );
        // assert_eq!(
        //     get_reserved(&mut grid, (2, 0).into()).unwrap(),
        //     ([PlanReserved::new(1, 8, u64::MAX)].as_slice()).into()
        // );

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        // assert_eq!(vehicle.lag_pos(tick), 0);
        assert_eq!(tick, 8);
        // assert_eq!(vehicle.pos, (2, 0).into());
        assert_eq!(vehicle.path_index, 3);
        assert_eq!(
            vehicle.reserved[2],
            PlanReservation::new((1, 0).into(), 8, 16)
        );
        assert_eq!(
            vehicle.reserved[1],
            PlanReservation::new((2, 0).into(), 16, 24)
        );
        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new((3, 0).into(), 24, u64::MAX)
        );
        assert_eq!(
            get_reserved(&mut grid, (3, 0).into()).unwrap(),
            ([PlanReserved::new(1, 24, u64::MAX)].as_slice()).into()
        );

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        assert_eq!(vehicle.pos, (2, 0).into());

        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new((4, 0).into(), 32, u64::MAX)
        );
    }

    #[test]
    fn intersection_traffic() {
        let mut grid = Grid::new_from_string(
            "LR>>1
             _^___",
        );

        let start: (Position, Direction) = ((1, 1).into(), Direction::UP);

        let mut vehicle = Vehicle::new(1, start, 1, &mut grid, 0).unwrap();

        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new(start.0, 0, u64::MAX)
        );
        assert_eq!(
            get_reserved(&mut grid, start.0).unwrap(),
            ([PlanReserved::new(vehicle.id, 0, u64::MAX)].as_slice()).into()
        );

        let _reservation = reserve(&mut grid, (1, 0).into(), 0, 0, 8).unwrap();
        vehicle.update(&mut grid, 0);
        assert_eq!(vehicle.path_index, 0);

        let tick = 4;

        vehicle.update(&mut grid, tick);

        assert_eq!(
            vehicle.reserved[2],
            PlanReservation::new((1, 1).into(), 0, 12)
        );
        assert_eq!(
            vehicle.reserved[1],
            PlanReservation::new((1, 0).into(), 12, 20)
        );
        assert_eq!(
            vehicle.reserved[0],
            PlanReservation::new((2, 0).into(), 20, u64::MAX)
        );

        assert_eq!(vehicle.path_index, 2);
    }

    #[test]
    fn yield_roundabout_traffic() {
        let mut grid = Grid::new_from_string(
            "\
            __.3__
            __.^__
            4<lr<<
            >>LR>2
            __.^__
            __1^__
            ",
        );

        let tick = 0;

        // // Top Vehicle is going straight down
        let start_top: (Position, Direction) = ((2, 1).into(), Direction::DOWN);
        let mut vehicle_top = Vehicle::new(1, start_top, 1, &mut grid, tick).unwrap();

        let start_left: (Position, Direction) = ((1, 3).into(), Direction::RIGHT);
        let mut vehicle_left = Vehicle::new(2, start_left, 2, &mut grid, tick).unwrap();

        let start_bottom: (Position, Direction) = ((3, 4).into(), Direction::UP);
        let mut vehicle_bottom = Vehicle::new(3, start_bottom, 3, &mut grid, tick).unwrap();

        let start_right: (Position, Direction) = ((4, 2).into(), Direction::LEFT);
        let mut vehicle_right = Vehicle::new(4, start_right, 4, &mut grid, tick).unwrap();

        // println!("grid: \n{:?}", grid);

        vehicle_top.update(&mut grid, 0);
        assert_eq!(vehicle_top.path_index, 3);
        vehicle_right.update(&mut grid, 1);
        assert_eq!(vehicle_right.path_index, 3);
        vehicle_bottom.update(&mut grid,2);
        assert_eq!(vehicle_bottom.path_index, 3);

        // This vehicle can't go through, because reservations are INCLUSIVE
        // So it just barely doesn't fit through.
        // Think of it like a garlic knot, in order for each peice to fit perfectly, there can't be any overlap
        // That's also why the update ticks above have to be incrementing.
        // dbg!(get_reserved(&mut grid, (2, 3).into()).unwrap());
        // dbg!(get_reserved(&mut grid, (3, 3).into()).unwrap());
        vehicle_left.update(&mut grid,1);
        assert_eq!(vehicle_left.path_index, 0);

        for tick in 0..SPEED_TICKS * 5 {
            vehicle_top.update(&mut grid, tick);
            vehicle_right.update(&mut grid, tick);
            vehicle_bottom.update(&mut grid, tick);
            vehicle_left.update(&mut grid, tick);
        }

        dbg!(&vehicle_top);
        assert_eq!(vehicle_top.update(&mut grid, 0), Status::ReachedDestination);
        assert_eq!(vehicle_right.update(&mut grid, 0), Status::ReachedDestination);
        assert_eq!(vehicle_bottom.update(&mut grid, 0), Status::ReachedDestination);
        // assert_eq!(vehicle_left.update(&mut grid, 0), Status::ReachedDestination);

        assert_eq!(vehicle_top.reserved.len(), 1);
        assert_eq!(vehicle_top.reserved.front().unwrap().pos, vehicle_top.pos);

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
