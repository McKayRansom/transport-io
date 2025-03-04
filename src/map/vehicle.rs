use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{
    grid::Grid,
    path::Path,
    position::GRID_CELL_SIZE,
    route::{RouteDir, RouteId},
    tile::{Reservation, ReservationError, RoadPosition, Tick, Tile},
    Direction, Position,
};

pub enum ReservePathError {
    InvalidPath,
    ReachedMaxLookahead,
    Blocking,
}

impl From<ReservationError> for ReservePathError {
    fn from(err: ReservationError) -> Self {
        match err {
            ReservationError::TileInvalid => ReservePathError::InvalidPath,
            ReservationError::TileReserved => ReservePathError::Blocking,
        }
    }
}

const SPEED_PIXELS: i32 = 4;
pub const SPEED_TICKS: Tick = ((GRID_CELL_SIZE.0 as Tick) / 2) / SPEED_PIXELS as Tick;
const HOPELESSLY_LATE_PERCENT: f32 = 0.01; // TODO: change path calc to account for busy streets...
const PATH_TIMEOUT: Tick = 8 * 20;

const RESERVE_AHEAD_MIN: usize = 8; // current tile + next tile
const RESERVE_AHEAD_MAX: usize = 16; // TODO: find correct

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Vehicle {
    pub pos: RoadPosition,
    pub dir: Direction,

    // this could be calculated from destination
    pub color: SpawnerColors,

    id: Id,
    pub grid_path: Path,
    pub reserved: VecDeque<Reservation>,
    path_index: usize,
    path_time_ticks: Tick,
    elapsed_ticks: Tick,
    pub destination: Id,
    pub route_info: Option<(RouteId, RouteDir)>,
    pub cargo: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd)]
pub enum Status {
    EnRoute,
    ReachedDestination,
    HopelesslyLate,
}

#[derive(Debug)]
pub enum VehicleError {
    InvalidTile,
    NoPath,
}

impl From<ReservationError> for VehicleError {
    fn from(_value: ReservationError) -> Self {
        VehicleError::InvalidTile
    }
}

impl Vehicle {
    pub fn new(
        id: Id,
        start: Position,
        destination: Id,
        grid: &mut Grid,
        tick: Tick, // maybe: Remove this, just reserve for all time (requires clean reservation state though...)
    ) -> Result<Self, VehicleError> {
        let path = grid
            .find_path(start, &destination)
            .ok_or(VehicleError::NoPath)?;

        // BUG: When starting from a multi-tile station, someone needs
        // to skip additional station tiles...
        // TODO: Cleanup when we convert path to a vec of directions!
        let start_dir = *path.0.get(1).ok_or(VehicleError::NoPath)? - start;
        let start_road_pos = RoadPosition::default_pos(start, start_dir);

        let reservation = Reservation::new(start_road_pos, tick, Tick::MAX);
        reservation.reserve(grid, id, tick)?;

        // TODO: TRY ROAD_POS ALT IF ONE_WAY DIR

        let mut vehicle = Self {
            pos: start_road_pos,
            dir: start_dir,
            color: SpawnerColors::Blue,

            id,
            grid_path: None,
            path_time_ticks: 0,
            path_index: 0,
            destination,
            elapsed_ticks: 0,
            reserved: VecDeque::from([reservation]),
            route_info: None,
            cargo: None,
        };

        vehicle.find_path(grid);
        // Err(VehicleError::NoPath)
        // } else {
        Ok(vehicle)
        // }
    }

    // TODO: Warn on drop if this isn't called...
    pub fn cleanup(&self, grid: &mut Grid) {
        for res in &self.reserved {
            let _ = res.unreserve(grid, self.id);
        }
    }

    pub fn lead_pos(&self, tick: Tick) -> i32 {
        // if we are stuck
        if self.reserved.len() <= 1 {
            return 0;
        }

        if let Some(reserve) = self.reserved.back() {
            // assert!(reserve.start > tick);
            if reserve.end == Tick::MAX {
                0
            } else {
                // This overflowing is a bug
                (tick - (reserve.end - SPEED_TICKS)) as i32 * SPEED_PIXELS
            }
        } else {
            0
        }
    }

    pub fn update(&mut self, grid: &mut Grid, tick: Tick) -> Status {
        self.update_trip();
        // if self.trip_late() < HOPELESSLY_LATE_PERCENT
        //     || self.grid_path.is_none() && self.elapsed_ticks > PATH_TIMEOUT
        // {
        //     return Status::HopelesslyLate;
        // }
        if let Some(res) = self.reserved.back() {
            if res.end > tick && res.end != Tick::MAX {
                return Status::EnRoute;
            }

            if res.end <= tick {
                let _ = self.reserved.pop_back().unwrap().unreserve(grid, self.id);
            }
        }

        self.update_position(grid, tick)
    }

    fn update_pos_dir(&mut self) {
        let mut iter = self.reserved.iter().rev();
        if let Some(res) = iter.next() {
            // self.dir = res.pos - self.pos;
            if let Some(res_next) = iter.next() {
                self.dir = res_next.pos - res.pos;
            }
            self.pos = res.pos;
        }
    }

    fn try_reserve_path(&mut self, grid: &mut Grid, tick: Tick) -> Result<(), ReservePathError> {
        if self.reserved.len() >= RESERVE_AHEAD_MIN {
            return Ok(());
        }

        let mut to_reserve: Vec<Reservation> = Vec::new();

        let mut start = tick;
        let mut reserved_head_pos = self.pos;

        if let Some(head) = self.reserved.front() {
            if head.end == Tick::MAX {
                start = head.start.max(tick) + SPEED_TICKS
            } else {
                start = head.end
            }
            reserved_head_pos = head.pos;
        } else {
            // BUG: If resereved is empty due to a different issue, we are unable to recover...
            // println!("WHAT");
        };

        let mut end = start + SPEED_TICKS;
        let mut last_road_pos = reserved_head_pos;

        // let mut intersection_count: usize = 0;

        'outer: for pos in &self
            .grid_path
            .as_ref()
            .ok_or(ReservePathError::InvalidPath)?
            .0[self.path_index..]
        {
            let mut one_way = false;
            if let Some(Tile::Road(road)) = grid.get_tile(&last_road_pos.pos) {
                if road.connections.is_one_way(*pos - last_road_pos.pos) {
                    one_way = true;
                }
            }

            for mut road_pos in last_road_pos.iter_succ(*pos, one_way) {
                let tile = grid
                    .get_tile(&road_pos.pos)
                    .ok_or(ReservePathError::InvalidPath)?;

                // Jankciy: Janktoberfest
                let mut tried_alt = false;
                if let Err(err) = tile.is_reserved(self.id, road_pos, start, end) {
                    if one_way {
                        road_pos = road_pos.one_way_alt(*pos - last_road_pos.pos);
                        tried_alt = true;
                        tile.is_reserved(self.id, road_pos, start, end)?;
                    } else {
                        return Err(err.into());
                    }
                }

                if tile.is_blockable() {
                    // we gotta check if we can for real stop here...
                    let end = Tick::MAX;

                    if let Err(err) = tile.is_reserved(self.id, road_pos, start, end) {
                        if one_way && !tried_alt {
                            road_pos = road_pos.one_way_alt(*pos - last_road_pos.pos);
                            tile.is_reserved(self.id, road_pos, start, end)?;
                        } else {
                            return Err(err.into());
                        }
                    }

                    to_reserve.push(Reservation::new(road_pos, start, end));

                    break 'outer;
                }

                to_reserve.push(Reservation::new(road_pos, start, end));
                last_road_pos = road_pos;

                start += SPEED_TICKS;
                end += SPEED_TICKS;

                if self.reserved.len() + to_reserve.len() > RESERVE_AHEAD_MAX {
                    return Err(ReservePathError::ReachedMaxLookahead);
                }
            }
        }

        // fixup the most recent (front) reservation to be the correct duration and not forever
        if !to_reserve.is_empty() {
            // this actually can happen if a road is delted
            let res = self
                .reserved
                .front_mut()
                .ok_or(ReservePathError::InvalidPath)?;

            if res.end == Tick::MAX {
                res.end = to_reserve[0].start;

                // this actually can happen if a road is delted
                if res.reserve(grid, self.id, tick).is_err() {
                    println!("THIS IS SUPER BAD!");
                }
            }
        }

        // if we've reached this point, we have a list of already free reservations to make
        for res in &to_reserve {
            // oh boy oh howdy, this is correct but feels jank
            if res.pos.pos != reserved_head_pos.pos {
                reserved_head_pos = res.pos;
                self.path_index += 1;
            }

            // if this fails, that means we messed up somewhere above
            res.reserve(grid, self.id, tick).unwrap();
            self.reserved.push_front(res.clone());
        }

        Ok(())
    }

    pub fn find_path(&mut self, grid: &Grid) -> bool {
        self.grid_path = grid.find_path(self.pos.pos, &self.destination);

        if let Some(path) = &self.grid_path {
            self.path_time_ticks = (path.1 + 1) * SPEED_TICKS;
            self.elapsed_ticks = 0;
            self.path_index = 0;
        }
        self.grid_path.is_some()
    }

    pub fn reserve_next_pos(&mut self, grid: &mut Grid, tick: Tick) {
        match self.try_reserve_path(grid, tick) {
            Ok(()) => {}
            Err(ReservePathError::InvalidPath) => {
                if self.path_time_ticks > 0 {
                    self.path_time_ticks = 0;
                    self.elapsed_ticks = 0;
                }
                self.find_path(grid);
            }
            Err(ReservePathError::Blocking) => {
                // self.blocking_tile = Some(blocking_pos);
            }
            Err(ReservePathError::ReachedMaxLookahead) => {
                // Might be nice if we could set the blocking tile to that position
                // Also, if we are currently stopped, this could mean an intersection
                // is too large to ever cross
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

        if let Some(tile) = grid.get_tile(&self.pos.pos) {
            if tile.get_building_id() == Some(self.destination) {
                return Status::ReachedDestination;
            }
        }

        self.reserve_next_pos(grid, tick);

        self.update_pos_dir();

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.grid_path {
            let tiles_elapsed = (self.elapsed_ticks.saturating_sub(1) / SPEED_TICKS) / 2 + 1;
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
    use crate::map::tile::{Reserved, ReservedList, Tile};

    use super::*;

    fn get_reserved(grid: &mut Grid, pos: RoadPosition) -> Result<ReservedList, ReservationError> {
        if let Some(Tile::Road(road)) = grid.get_tile(&pos.pos) {
            Ok(road.reservations[pos.index()].clone())
        } else {
            Err(ReservationError::TileInvalid)
        }
    }

    #[test]
    fn test_init() {
        let mut grid = Grid::new_from_string(">>>1");
        let start_road_pos = RoadPosition::default_pos((0, 0).into(), Direction::RIGHT);
        let vehicle = Vehicle::new(1, start_road_pos.pos, 1, &mut grid, 0).unwrap();

        assert!(Vehicle::new(2, start_road_pos.pos, 1, &mut grid, 0).is_err());

        assert_eq!(
            vehicle.reserved,
            [Reservation::new(start_road_pos, 0, Tick::MAX)]
        );

        assert_eq!(
            get_reserved(&mut grid, start_road_pos).unwrap(),
            ([Reserved::new(vehicle.id, 0, Tick::MAX)].as_slice()).into()
        );

        assert_eq!(vehicle.lead_pos(0), 0);
    }

    #[test]
    #[ignore = "reason"]
    fn test_lead_pos() {
        let mut grid = Grid::new_from_string(">>>>1");
        let mut vehicle = Vehicle::new(1, (0, 0).into(), 1, &mut grid, 0).unwrap();

        for tick in 0..16 {
            assert_eq!(vehicle.update(&mut grid, tick), Status::EnRoute);
            assert_eq!(vehicle.path_index as Tick, tick / SPEED_TICKS, "i: {tick}");
            assert_eq!(
                vehicle.pos,
                (
                    ((tick as i16 / SPEED_TICKS as i16) / 2, 0),
                    (((tick as i16 / SPEED_TICKS as i16) % 2) as i8, 1)
                )
                    .into(),
                "i: {tick}"
            );
            assert_eq!(
                vehicle.pos,
                vehicle.reserved.back().unwrap().pos,
                "i: {tick}"
            );
            assert_eq!(
                vehicle.lead_pos(tick),
                (tick as i32 * SPEED_PIXELS) % GRID_CELL_SIZE.0 as i32,
                "i: {tick}"
            );
            println!("{}: {:?} + {}", tick, vehicle.pos, vehicle.lead_pos(tick));
        }
    }

    #[test]
    fn test_straight() {
        let mut grid = Grid::new_from_string(">>>>1");
        let mut vehicle = Vehicle::new(42, (0, 0).into(), 1, &mut grid, 0).unwrap();

        vehicle.update(&mut grid, 0);

        // assert_eq!(
        //     vehicle.reserved,
        //     [
        //         Reservation::new(((0, 0), (1, 1)).into(), SPEED_TICKS, Tick::MAX),
        //         Reservation::new(((0, 0), (0, 1)).into(), 0, SPEED_TICKS),
        //     ]
        // );

        // vehicle.update(&mut grid, SPEED_TICKS);

        assert_eq!(
            vehicle.reserved,
            [
                Reservation::new(((1, 0), (0, 1)).into(), SPEED_TICKS, Tick::MAX),
                Reservation::new(((0, 0), (1, 1)).into(), 0, SPEED_TICKS),
            ]
        );

        // assert!(Reservation::new((0, 0).into(), 0, 1)
        //     .is_reserved(&mut grid, 1234)
        //     .is_err());

        vehicle.update(&mut grid, SPEED_TICKS * 2);

        assert_eq!(
            vehicle.reserved,
            [
                Reservation::new(((1, 0), (1, 1)).into(), SPEED_TICKS * 3, Tick::MAX),
                Reservation::new(((1, 0), (0, 1)).into(), SPEED_TICKS, SPEED_TICKS * 3),
            ]
        );
    }

    #[test]
    // #[ignore = "reason"]
    fn test_late() {
        let mut grid = Grid::new_from_string(">>>1");
        let mut vehicle = Vehicle::new(1, (0, 0).into(), 1, &mut grid, 0).unwrap();

        vehicle.update(&mut grid, 0);

        vehicle.elapsed_ticks = SPEED_TICKS + 1;
        assert_eq!(vehicle.trip_late(), 1.0);
        vehicle.elapsed_ticks = SPEED_TICKS * 2 + 1;
        assert_eq!(vehicle.trip_late(), 0.76);

        vehicle.elapsed_ticks = (SPEED_TICKS * 4) + 1;
        assert_eq!(vehicle.trip_late(), 0.5);
    }

    #[test]
    #[ignore = "reason"]
    fn test_trip() {
        let mut grid = Grid::new_from_string(">>>1");
        let mut vehicle = Vehicle::new(1, (0, 0).into(), 1, &mut grid, 0).unwrap();

        let trip_length: u32 = 3;
        let trip_time = SPEED_TICKS * trip_length;

        assert_eq!(vehicle.path_time_ticks, trip_time);
        assert_eq!(vehicle.trip_completed_percent(), 0.);
        assert_eq!(vehicle.trip_late(), 1.0);

        for i in 0..(trip_length * SPEED_TICKS) {
            assert_eq!(
                vehicle.update(&mut grid, i as Tick),
                Status::EnRoute,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.path_index,
                (i / (SPEED_TICKS)) as usize,
                "Failed on tick {i}"
            );
            assert_eq!(vehicle.elapsed_ticks, i + 1);
            assert_eq!(
                vehicle.trip_completed_percent(),
                ((i * SPEED_TICKS) / SPEED_TICKS) as f32 / trip_length as f32,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.trip_late(),
                1.0,
                "Failed on tick {i} %{}",
                vehicle.trip_completed_percent()
            );
        }
    }

    #[test]
    #[ignore = "reason"]
    fn instersection_do_not_block() {
        let mut grid = Grid::new_from_string(
            "LR>>1
             _^___",
        );

        let start: Position = (1, 1).into();
        let start_road_pos: RoadPosition = RoadPosition::default_pos(start, Direction::UP);

        let mut vehicle = Vehicle::new(1, start, 1, &mut grid, 0).unwrap();

        assert_eq!(
            vehicle.reserved[0],
            Reservation::new(start_road_pos, 0, Tick::MAX)
        );

        // reserve the exit, make sure we don't go anyways
        let reservation = Reservation::new((2, 0).into(), 0, 16);
        reservation.reserve(&mut grid, 1234, 0).unwrap();

        vehicle.update(&mut grid, 0);

        assert_eq!(vehicle.path_index, 0);
        assert_eq!(vehicle.pos, start_road_pos);

        reservation.unreserve(&mut grid, 1234).unwrap();

        let mut tick = 0;

        vehicle.update(&mut grid, tick);

        assert_eq!(
            vehicle.reserved,
            [
                Reservation::new((2, 0).into(), 16, Tick::MAX),
                Reservation::new((1, 0).into(), 8, 16),
                Reservation::new((1, 1).into(), 0, 8),
            ]
        );

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        assert_eq!(tick, 8);
        assert_eq!(vehicle.path_index, 2);

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        assert_eq!(tick, 16);

        assert_eq!(
            vehicle.reserved,
            [
                Reservation::new((3, 0).into(), 24, Tick::MAX),
                Reservation::new((2, 0).into(), 16, 24),
            ]
        );
        assert_eq!(
            get_reserved(&mut grid, (3, 0).into()).unwrap(),
            ([Reserved::new(1, 24, Tick::MAX)].as_slice()).into()
        );

        for _ in 0..SPEED_TICKS {
            tick += 1;
            vehicle.update(&mut grid, tick);
        }

        assert_eq!(
            vehicle.reserved,
            [
                Reservation::new((4, 0).into(), 32, Tick::MAX),
                Reservation::new((3, 0).into(), 24, 32),
            ]
        );
    }

    #[test]
    #[ignore = "I don't want to"]
    fn intersection_overlap_traffic() {
        let mut grid = Grid::new_from_string(
            "LR>>1
             _^___",
        );

        let start: Position = (1, 1).into();
        let start_road_pos: RoadPosition = RoadPosition::default_pos(start, Direction::UP);

        let mut vehicle = Vehicle::new(1, start, 1, &mut grid, 0).unwrap();

        assert_eq!(
            vehicle.reserved,
            [Reservation::new(start_road_pos, 0, Tick::MAX)]
        );

        let reservation = Reservation::new((1, 0).into(), 0, 8);
        reservation.reserve(&mut grid, 1234, 0).unwrap();

        vehicle.update(&mut grid, 0);

        assert_eq!(
            vehicle.reserved,
            [Reservation::new(start_road_pos, 0, Tick::MAX)]
        );

        vehicle.update(&mut grid, 4);

        assert_eq!(
            vehicle.reserved,
            [
                Reservation::new((2, 0).into(), 20, Tick::MAX),
                Reservation::new((1, 0).into(), 12, 20),
                Reservation::new((1, 1).into(), 0, 12),
            ]
        );
    }

    #[test]
    fn yield_roundabout_traffic() {
        let mut grid = Grid::new_from_string(
            "\
            __3__
            __|__
            4-+-2
            __|__
            __1__
            ",
        );

        let tick = 0;

        // // Top Vehicle is going straight down
        let mut vehicle_top = Vehicle::new(1, (2, 1).into(), 1, &mut grid, tick).unwrap();
        let mut vehicle_left = Vehicle::new(2, (1, 2).into(), 2, &mut grid, tick).unwrap();
        let mut vehicle_bottom = Vehicle::new(3, (2, 3).into(), 3, &mut grid, tick).unwrap();
        let mut vehicle_right = Vehicle::new(4, (3, 2).into(), 4, &mut grid, tick).unwrap();

        vehicle_top.update(&mut grid, 0);
        assert_eq!(vehicle_top.path_index, 2);
        vehicle_right.update(&mut grid, 1);
        assert_eq!(vehicle_right.path_index, 2);
        vehicle_bottom.update(&mut grid, 2);
        assert_eq!(vehicle_bottom.path_index, 2);

        // This vehicle can't go through, because reservations are INCLUSIVE
        // So it just barely doesn't fit through.
        // Think of it like a garlic knot, in order for each peice to fit perfectly, there can't be any overlap
        // That's also why the update ticks above have to be incrementing.
        vehicle_left.update(&mut grid, 1);
        assert_eq!(vehicle_left.path_index, 0);

        for tick in 0..SPEED_TICKS * 6 {
            vehicle_top.update(&mut grid, tick);
            vehicle_right.update(&mut grid, tick);
            vehicle_bottom.update(&mut grid, tick);
            vehicle_left.update(&mut grid, tick);
        }

        dbg!(&vehicle_top);
        assert_eq!(vehicle_top.update(&mut grid, 0), Status::ReachedDestination);
        assert_eq!(
            vehicle_right.update(&mut grid, 0),
            Status::ReachedDestination
        );
        assert_eq!(
            vehicle_bottom.update(&mut grid, 0),
            Status::ReachedDestination
        );

        assert_eq!(vehicle_top.reserved.len(), 1);
        assert_eq!(vehicle_top.reserved.front().unwrap().pos, vehicle_top.pos);
    }

    #[test]
    fn house_yield() {
        let mut grid = Grid::new_from_string(
            ">>>2
             _h__", // house is id 1
        );

        let mut vehicle_priority = Vehicle::new(42, (1, 0).into(), 2, &mut grid, 0).unwrap();

        vehicle_priority.update(&mut grid, 0);

        assert_eq!(
            vehicle_priority.reserved,
            [
                Reservation::new(((2, 0), (0, 1)).into(), SPEED_TICKS, Tick::MAX),
                Reservation::new(((1, 0), (1, 1)).into(), 0, SPEED_TICKS),
            ]
        );

        let mut vehicle_yield = Vehicle::new(2, (1, 1).into(), 2, &mut grid, 0).unwrap();

        vehicle_yield.update(&mut grid, 0);

        assert_eq!(vehicle_yield.path_index, 0);
    }
}
