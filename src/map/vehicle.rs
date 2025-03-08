use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{
    grid::Grid,
    path::Path,
    position::GRID_CELL_SIZE,
    route::{RouteDir, RouteId},
    tile::{Reservation, ReservationError, RoadPosition, Tile},
    Direction, Position, Tick,
};

pub enum ReservePathError {
    InvalidPath,
    // ReachedMaxLookahead,
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
const ROAD_TILE_PIXELS: i32 = (GRID_CELL_SIZE.0 as i32) / 2;
pub const SPEED_TICKS: Tick = ((GRID_CELL_SIZE.0 as Tick) / 2) / SPEED_PIXELS as Tick;
const HOPELESSLY_LATE_PERCENT: f32 = 0.01; // TODO: change path calc to account for busy streets...
const PATH_TIMEOUT: Tick = 8 * 20;

// const RESERVE_AHEAD_MIN: usize = 8; // current tile + next tile
// const RESERVE_AHEAD_MAX: usize = 16; // TODO: find correct

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Vehicle {
    pub pos: RoadPosition,
    pub dir: Direction,
    pub lag_pos: i32,
    // pub lag_speed: u32,

    // this could be calculated from destination
    pub color: SpawnerColors,

    id: Id,
    pub grid_path: Path,
    pub reserved: Vec<Reservation>,
    path_index: usize,
    path_time_ticks: Tick,
    elapsed_ticks: Tick,
    pub destination: Id,
    // TEMP: HACK USE RouteId as Starting building ID!
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
    OccupiedTile,
    NoPath,
}

impl From<ReservationError> for VehicleError {
    fn from(_value: ReservationError) -> Self {
        VehicleError::OccupiedTile
    }
}

impl Vehicle {
    pub fn new(
        id: Id,
        start: Position,
        destination: Id,
        grid: &mut Grid,
    ) -> Result<Self, VehicleError> {
        let path = grid
            .find_path(start, &destination)
            .ok_or(VehicleError::NoPath)?;

        // BUG: When starting from a multi-tile station, someone needs
        // to skip additional station tiles...
        // TODO: Cleanup when we convert path to a vec of directions!
        let start_dir = *path.0.get(1).ok_or(VehicleError::NoPath)? - start;
        let start_road_pos = RoadPosition::default_pos(start, start_dir);

        let start_tile = grid
            .get_tile_mut(&start_road_pos.pos)
            .ok_or(VehicleError::InvalidTile)?;

        let reservation = start_tile.reserve(id, start_road_pos)?;

        // TODO: TRY ROAD_POS ALT IF ONE_WAY DIR

        let path_time_ticks = (path.1 + 1) * SPEED_TICKS; // TODO :De-dup this

        Ok(Self {
            pos: start_road_pos,
            dir: start_dir,
            lag_pos: 0,
            color: SpawnerColors::Blue,

            id,
            grid_path: Some(path),
            path_time_ticks,
            path_index: 0,
            destination,
            elapsed_ticks: 0,
            reserved: Vec::from([reservation]),
            route_info: None,
            cargo: None,
        })
    }

    pub fn fixup(&mut self, grid: &mut Grid) -> Result<(), ReservationError> {
        // Fix serialization
        // BUG: When reserializing, we fd this up when we allowed left turns to get BuildingReservations...
        // For now we can create a building reservation but this is not deterministic...
        for reservation in &mut self.reserved {
            // if reservation.
            *reservation = grid
                .get_tile_mut(&reservation.pos.pos)
                .ok_or(ReservationError::TileInvalid)?
                .reserve(self.id, reservation.pos)
                .unwrap_or_else(|_err| Reservation::new_for_building(reservation.pos));
        }
        Ok(())
    }

    fn update_speed(&mut self) {
        if self.lag_pos > 0 {
            self.lag_pos -= SPEED_PIXELS; // (self.lag_speed / 4).min(self.lag_pos);
        }
        // if self.lag_speed < SPEED_PIXELS_PER_TICK {
        //     self.lag_speed += ACCEL_PIXELS_PER_TICK;
        // }
    }

    pub fn update(&mut self, grid: &mut Grid, _tick: Tick) -> Status {
        self.update_trip();
        if self.trip_late() < HOPELESSLY_LATE_PERCENT
            || self.grid_path.is_none() && self.elapsed_ticks > PATH_TIMEOUT
        {
            return Status::HopelesslyLate;
        }
        // if let Some(res) = self.reserved.back() {
        //     if res.end > tick && res.end != Tick::MAX {
        //         return Status::EnRoute;
        //     }

        //     if res.end <= tick {
        //         let _ = self.reserved.pop_back().unwrap().unreserve(grid, self.id);
        //     }
        // }

        self.update_speed();
        if self.lag_pos != 0 {
            Status::EnRoute
        } else {
            self.update_position(grid)
        }
    }
    // TODO
    // pub fn update_next_pos(&mut self, pos: Option<Position>) {
    //     if let Some(next_pos) = pos {
    //         self.dir = next_pos - self.grid_pos;
    //         // self.dir.z = -self.dir.z;
    //         self.lag_pos = GRID_CELL_SIZE.0 as u32;
    //         self.update_speed();
    //         self.grid_pos = next_pos;
    //     } else {
    //         self.lag_speed = 0;
    //     }
    // }

    fn update_pos_dir(&mut self) {
        // let mut iter = self.reserved.iter();
        if let Some(res) = self.reserved.first() {
            self.dir = res.pos - self.pos;
            // if let Some(res_next) = iter.next() {
            //     self.dir = res_next.pos - res.pos;
            // }
            if res.pos.pos != self.pos.pos {
                self.path_index += 1; // OOOOFFFF
            }
            self.pos = res.pos;
            self.lag_pos = ROAD_TILE_PIXELS;
        }
    }

    fn try_reserve_path(&mut self, grid: &mut Grid) -> Result<Vec<Reservation>, ReservePathError> {
        let mut to_reserve: Vec<Reservation> = Vec::new();
        let mut last_road_pos = self.pos;
        let mut start_house = false;
        let mut end_house = false;

        // TODO: convert path to VecDequeue
        // TODO: Is there a path_index off-by-one error?
        'outer: for pos in &self
            .grid_path
            .as_ref()
            .ok_or(ReservePathError::InvalidPath)?
            .0[self.path_index..]
        {
            let dir = *pos - last_road_pos.pos;
            let mut one_way = false;
            match grid.get_tile(&last_road_pos.pos) {
                Some(Tile::Road(road)) => {
                    if road.connections.is_one_way(dir) {
                        one_way = true;
                    }
                }
                Some(Tile::Building(_)) => {
                    if self.path_index == 0 {
                        start_house = true;
                    }
                }
                _ => {}
            }

            // TODO: Check if end of path
            if self.path_index > 0 {
                if let Some(Tile::Building(_)) = grid.get_tile(pos) {
                    end_house = true;
                }
            }

            assert!(dir.x.abs() < 2 && dir.y.abs() < 2, "vehicle: {self:?}");

            for road_pos in last_road_pos.iter_succ(*pos, one_way) {
                let tile = grid
                    .get_tile_mut(&road_pos.pos)
                    .ok_or(ReservePathError::InvalidPath)?;

                // Jankciy: Janktoberfest
                match tile.reserve(self.id, road_pos) {
                    Ok(res) => to_reserve.push(res),
                    Err(err) => {
                        if end_house {
                            to_reserve.push(Reservation::new_for_building(road_pos));
                        } else if one_way && tile.is_blockable() {
                            if let Some(alt_pos) = road_pos.one_way_alt(dir) {
                                // println!("Found one-way alt: {alt_pos:?} for: {road_pos:?} dir: {dir:?}");
                                // NOTE: adding intermediate pos doesn't work because we re-reserve every tile...
                                // In general I'm not convinced it would work well but I'm not sure...
                                // to_reserve.push(Reservation::new_for_building(road_pos));
                                to_reserve.push(tile.reserve(self.id, alt_pos)?);
                            } else {
                                return Err(err.into());
                            }
                        } else {
                            return Err(err.into());
                        }
                    }
                }

                if tile.is_blockable() && !start_house {
                    break 'outer;
                }

                start_house = false;
                last_road_pos = road_pos;
            }
            end_house = false;
        }

        Ok(to_reserve)
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

    pub fn reserve_next_pos(&mut self, grid: &mut Grid) {
        match self.try_reserve_path(grid) {
            Ok(reserved) => {
                self.reserved = reserved.into();

                self.update_pos_dir();
            }
            Err(ReservePathError::InvalidPath) => {
                if self.path_time_ticks > 0 {
                    self.path_time_ticks = 0;
                    self.elapsed_ticks = 0;
                }
                self.find_path(grid);
            }
            Err(ReservePathError::Blocking) => {
                // self.blocking_tile = Some(blocking_pos);
            } // Err(ReservePathError::ReachedMaxLookahead) => {
              // Might be nice if we could set the blocking tile to that position
              // Also, if we are currently stopped, this could mean an intersection
              // is too large to ever cross
              // }
        }
    }

    pub fn update_trip(&mut self) {
        self.elapsed_ticks += 1;
    }

    pub fn update_position(&mut self, grid: &mut Grid) -> Status {
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

        self.reserve_next_pos(grid);

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        let Some(path) = &self.grid_path else {
            return 1.;
        };

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
    use crate::map::tile::{Reserved, Tile};

    use super::*;

    fn _get_reserved(grid: &mut Grid, pos: RoadPosition) -> Result<Reserved, ReservationError> {
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
        let vehicle = Vehicle::new(1, start_road_pos.pos, 1, &mut grid).unwrap();

        assert!(Vehicle::new(2, start_road_pos.pos, 1, &mut grid).is_err());

        // assert_eq!(
        //     vehicle.reserved.f,
        //     [Reservation::new(start_road_pos)]
        // );

        // assert_eq!(
        //     get_reserved(&mut grid, start_road_pos).unwrap(),
        //     ([Reserved::new(vehicle.id, 0, Tick::MAX)].as_slice()).into()
        // );

        assert_eq!(vehicle.lag_pos, 0);
    }

    #[test]
    fn test_lead_pos() {
        let mut grid = Grid::new_from_string(">>>>1");
        let mut vehicle = Vehicle::new(1, (0, 0).into(), 1, &mut grid).unwrap();

        for tick in 0..16 {
            assert_eq!(vehicle.update(&mut grid, tick), Status::EnRoute);
            assert_eq!(
                vehicle.path_index as Tick,
                (tick / SPEED_TICKS) / 2 + 1,
                "i: {tick} {}",
                vehicle.lag_pos
            );
            assert_eq!(
                vehicle.pos,
                (
                    ((tick as i16 / SPEED_TICKS as i16) / 2 + 1, 0),
                    (((tick as i16 / SPEED_TICKS as i16) % 2) as i8, 1)
                )
                    .into(),
                "i: {tick}"
            );
            assert_eq!(
                vehicle.pos,
                vehicle.reserved.first().unwrap().pos,
                "i: {tick}"
            );
            assert_eq!(
                vehicle.lag_pos,
                ROAD_TILE_PIXELS - (tick as i32 * SPEED_PIXELS) % ROAD_TILE_PIXELS,
                "i: {tick}"
            );
            println!("{}: {:?} + {}", tick, vehicle.pos, vehicle.lag_pos);
        }
    }

    // #[test]
    // // #[ignore = "reason"]
    // fn test_late() {
    //     let mut grid = Grid::new_from_string(">>>1");
    //     let mut vehicle = Vehicle::new(1, (0, 0).into(), 1, &mut grid, 0).unwrap();

    //     vehicle.update(&mut grid, 0);

    //     vehicle.elapsed_ticks = SPEED_TICKS + 1;
    //     assert_eq!(vehicle.trip_late(), 1.0);
    //     vehicle.elapsed_ticks = SPEED_TICKS * 2 + 1;
    //     assert_eq!(vehicle.trip_late(), 0.76);

    //     vehicle.elapsed_ticks = (SPEED_TICKS * 4) + 1;
    //     assert_eq!(vehicle.trip_late(), 0.5);
    // }

    // #[test]
    // #[ignore = "reason"]
    // fn test_trip() {
    //     let mut grid = Grid::new_from_string(">>>1");
    //     let mut vehicle = Vehicle::new(1, (0, 0).into(), 1, &mut grid, 0).unwrap();

    //     let trip_length: u32 = 3;
    //     let trip_time = SPEED_TICKS * trip_length;

    //     assert_eq!(vehicle.path_time_ticks, trip_time);
    //     assert_eq!(vehicle.trip_completed_percent(), 0.);
    //     assert_eq!(vehicle.trip_late(), 1.0);

    //     for i in 0..(trip_length * SPEED_TICKS) {
    //         assert_eq!(
    //             vehicle.update(&mut grid, i as Tick),
    //             Status::EnRoute,
    //             "Failed on tick {i}"
    //         );
    //         assert_eq!(
    //             vehicle.path_index,
    //             (i / (SPEED_TICKS)) as usize,
    //             "Failed on tick {i}"
    //         );
    //         assert_eq!(vehicle.elapsed_ticks, i + 1);
    //         assert_eq!(
    //             vehicle.trip_completed_percent(),
    //             ((i * SPEED_TICKS) / SPEED_TICKS) as f32 / trip_length as f32,
    //             "Failed on tick {i}"
    //         );
    //         assert_eq!(
    //             vehicle.trip_late(),
    //             1.0,
    //             "Failed on tick {i} %{}",
    //             vehicle.trip_completed_percent()
    //         );
    //     }
    // }

    #[test]
    fn instersection_do_not_block() {
        let mut grid = Grid::new_from_string(
            "L+>>1
             _^___",
        );

        let start: Position = (1, 1).into();
        let start_road_pos: RoadPosition = RoadPosition::default_pos(start, Direction::UP);

        let mut vehicle = Vehicle::new(1, start, 1, &mut grid).unwrap();

        // reserve the exit, make sure we don't go anyways
        let reservation = grid
            .get_tile_mut(&(2, 0).into())
            .unwrap()
            .reserve(1234, RoadPosition::new((2, 0), (0, 1)));

        vehicle.update(&mut grid, 0);

        for _ in 0..16 {
            vehicle.update(&mut grid, 0);
        }

        assert_eq!(vehicle.path_index, 0);
        assert_eq!(vehicle.pos, start_road_pos);

        drop(reservation);

        vehicle.update(&mut grid, 0);

        assert_eq!(vehicle.path_index, 1);
        assert_eq!(vehicle.pos, start_road_pos + Direction::UP);

        for _ in 0..128 {
            vehicle.update(&mut grid, 0);
        }
    }

    // #[test]
    // #[ignore = "I don't want to"]
    // fn intersection_overlap_traffic() {
    //     let mut grid = Grid::new_from_string(
    //         "LR>>1
    //          _^___",
    //     );

    //     let start: Position = (1, 1).into();
    //     let start_road_pos: RoadPosition = RoadPosition::default_pos(start, Direction::UP);

    //     let mut vehicle = Vehicle::new(1, start, 1, &mut grid, 0).unwrap();

    //     assert_eq!(
    //         vehicle.reserved,
    //         [Reservation::new(start_road_pos, 0, Tick::MAX)]
    //     );

    //     let reservation = Reservation::new((1, 0).into(), 0, 8);
    //     reservation.reserve(&mut grid, 1234, 0).unwrap();

    //     vehicle.update(&mut grid, 0);

    //     assert_eq!(
    //         vehicle.reserved,
    //         [Reservation::new(start_road_pos, 0, Tick::MAX)]
    //     );

    //     vehicle.update(&mut grid, 4);

    //     assert_eq!(
    //         vehicle.reserved,
    //         [
    //             Reservation::new((2, 0).into(), 20, Tick::MAX),
    //             Reservation::new((1, 0).into(), 12, 20),
    //             Reservation::new((1, 1).into(), 0, 12),
    //         ]
    //     );
    // }

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

        // Top Vehicle is going straight down
        let mut vehicle_top = Vehicle::new(1, (2, 1).into(), 1, &mut grid).unwrap();
        let mut vehicle_left = Vehicle::new(2, (1, 2).into(), 2, &mut grid).unwrap();
        let mut vehicle_bottom = Vehicle::new(3, (2, 3).into(), 3, &mut grid).unwrap();
        let mut vehicle_right = Vehicle::new(4, (3, 2).into(), 4, &mut grid).unwrap();

        vehicle_top.update(&mut grid, 0);
        assert_eq!(vehicle_top.path_index, 1);
        vehicle_right.update(&mut grid, 1);
        assert_eq!(vehicle_right.path_index, 1);
        vehicle_bottom.update(&mut grid, 2);
        assert_eq!(vehicle_bottom.path_index, 1);

        // This vehicle can't go through, because reservations are INCLUSIVE
        // So it just barely doesn't fit through.
        // Think of it like a garlic knot, in order for each peice to fit perfectly, there can't be any overlap
        // That's also why the update ticks above have to be incrementing.
        vehicle_left.update(&mut grid, 1);
        assert_eq!(vehicle_left.path_index, 1);

        for tick in 0..SPEED_TICKS * 6 {
            vehicle_top.update(&mut grid, tick);
            vehicle_right.update(&mut grid, tick);
            vehicle_bottom.update(&mut grid, tick);
            vehicle_left.update(&mut grid, tick);
        }

        assert_eq!(vehicle_top.update(&mut grid, 0), Status::ReachedDestination);
        // assert_eq!(
        //     vehicle_right.update(&mut grid, 0),
        //     Status::ReachedDestination
        // );
        // assert_eq!(
        //     vehicle_bottom.update(&mut grid, 0),
        //     Status::ReachedDestination
        // );

        // assert_eq!(vehicle_top.reserved.len(), 1);
        // assert_eq!(vehicle_top.reserved.first().unwrap().pos, vehicle_top.pos);
    }

    #[test]
    fn house_yield() {
        let mut grid = Grid::new_from_string(
            ">>>2
             _h__", // house is id 1
        );

        let res_pos = RoadPosition::new((1, 0), (1, 1));
        let reservation = grid
            .get_tile_mut(&res_pos.pos)
            .unwrap()
            .reserve(1234, res_pos);

        let mut vehicle_yield =
            Vehicle::new(2, RoadPosition::new((1, 1), (1, 0)).into(), 2, &mut grid).unwrap();

        vehicle_yield.update(&mut grid, 0);
        assert_eq!(vehicle_yield.path_index, 0);

        drop(reservation);

        vehicle_yield.update(&mut grid, 0);
        assert_eq!(vehicle_yield.path_index, 1);
    }

    #[test]
    fn house_left_turn() {
        let mut grid = Grid::new_from_string(
            "<<<
             _h_", // house is id 1
        );

        let res_pos = RoadPosition::new((1, 0), (0, 1));
        let reservation = grid
            .get_tile_mut(&res_pos.pos)
            .unwrap()
            .reserve(1234, res_pos);

        let mut vehicle_priority =
            Vehicle::new(2, RoadPosition::new((2, 0), (0, 0)).into(), 1, &mut grid).unwrap();

        for _ in 0..32 {
            vehicle_priority.update(&mut grid, 0);
        }
        assert_eq!(
            vehicle_priority.update(&mut grid, 0),
            Status::ReachedDestination
        );
        assert_eq!(vehicle_priority.path_index, 2);

        drop(reservation);
    }
}
