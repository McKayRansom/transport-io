use macroquad::color::Color;
use macroquad::color::BLUE;
use macroquad::color::RED;
use macroquad::color::WHITE;
use macroquad::math::Rect;

use crate::grid::Direction;
use crate::grid::Grid;
use crate::grid::Path;
use crate::grid::Position;
use crate::grid::ReservationStatus;
use crate::tile::ConnectionLayer;
use crate::tile::Tile;
use crate::grid::GRID_CELL_SIZE;
use crate::tileset::Tileset;

use crate::grid::Id;

const SPEED_PIXELS_PER_TICK: i16 = 4;
const SPEED_TICKS_PER_TILE: i16 = GRID_CELL_SIZE.0 as i16 / SPEED_PIXELS_PER_TICK;
const HOPELESSLY_LATE_PERCENT: f32 = 0.5;

pub struct Vehicle {
    pub id: Id,
    path: Path,
    path_index: usize,
    path_time_ticks: u32,
    elapsed_ticks: u32,
    pos: Position,
    lag_pos_pixels: i16,
    dir: Direction,
    // station_id: usize
    blocking_tile: Option<Position>,
    pub destination: Position,
    reserved: Option<Vec<Position>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd)]
pub enum Status {
    EnRoute,
    ReachedDestination,
    HopelesslyLate,
}

pub enum ReservePathStatus {
    InvalidPath,
    Success(Vec<Position>),
    Blocking(Position),
}

pub enum ShouldWeYieldStatus {
    Yield(Position),
    Clear,
}

impl Vehicle {
    pub fn new(pos: Position, id: Id, destination: Position, grid: &mut Grid) -> Option<Self> {
        if grid.reserve_position(&pos, id) != ReservationStatus::TileSuccess {
            return None;
        }

        let mut vehicle = Vehicle {
            id: id,
            path: None,
            path_time_ticks: 0,
            path_index: 0,
            elapsed_ticks: 0,
            pos: pos,
            lag_pos_pixels: 0,
            dir: Direction::Right,
            blocking_tile: None,
            destination: destination,
            reserved: Some(vec![pos]),
        };

        vehicle.find_path(grid);

        Some(vehicle)
    }

    pub fn delete(&mut self, grid: &mut Grid) {
        grid.unreserve_position(&self.pos);
        if let Some(reserved) = &mut self.reserved {
            Vehicle::clear_reserved(grid, reserved);
        }
    }

    fn reserve(
        path_grid: &mut Grid,
        vehicle_id: Id,
        position: Position,
        reserved: &mut Vec<Position>,
    ) -> ReservationStatus {
        let status = path_grid.reserve_position(&position, vehicle_id);
        if status == ReservationStatus::TileSuccess {
            reserved.push(position);
        } else {
            Vehicle::clear_reserved(path_grid, reserved);
        }
        status
    }

    pub fn clear_reserved(path_grid: &mut Grid, reserved: &mut Vec<Position>) {
        for i in 0..reserved.len() {
            path_grid.unreserve_position(&reserved[i]);
        }
        reserved.clear();
    }

    fn should_we_yield_when_entering(
        &self,
        path_grid: &Grid,
        my_tile: &Tile,
        position: &Position,
    ) -> ShouldWeYieldStatus {
        // never yield from an intersection
        if let Tile::Road(road) = my_tile {
            if road.connection_count() > 1 {
                return ShouldWeYieldStatus::Clear;
            }
        }

        if let Tile::Road(road) = path_grid.get_tile(position) {
            // For each direction that feeds into this tile in question
            for dir in road
                .iter_connections_inverse(ConnectionLayer::Road)
            {
                if let Some(yield_to_pos) = Position::new_from_move(position, dir, path_grid.size) {

                // Get the road
                if let Tile::Road(yield_to_road) = path_grid.get_tile(&yield_to_pos) {
                    // if it's reserved and connects to our road
                    if yield_to_road.reserved.is_some()
                        && yield_to_road.is_connected(dir.inverse())
                    {
                        match my_tile {
                            // alway yield from a house
                            Tile::House(_) => {
                                return ShouldWeYieldStatus::Yield(yield_to_pos);
                            }
                            // if we are somehow in a weird state, I guess yield?
                            Tile::Empty => {
                                return ShouldWeYieldStatus::Yield(yield_to_pos);
                            }
                            Tile::Road(_) => {
                                // Always yield when entering an interseciton (if we aren't an intersection)
                                // because of the check above we know we are NOT an intersection
                                if yield_to_road.connection_count() > 1 {
                                    return ShouldWeYieldStatus::Yield(yield_to_pos);
                                }
                                // otherwise if the tile is a normal road we probably don't need to yield
                            }
                        }
                    }
                }
            }
            }
        }

        return ShouldWeYieldStatus::Clear;
    }

    fn reserve_path(&self, path_grid: &mut Grid) -> ReservePathStatus {
        // let mut should_yield = path_grid.should_yield(&self.pos);
        let my_tile = *path_grid.get_tile(&self.pos);
        let mut reserved = Vec::<Position>::new();
        // println!("should_yield: {}", should_yield);
        if self.path.is_none() {
            return ReservePathStatus::InvalidPath;
        }

        // TODO: Remove clone
        let path: &Vec<Position> = &self.path.clone().unwrap().0;

        for i in self.path_index..path.len() {
            let pos = path.get(i).unwrap();
            if *pos == self.pos {
                continue;
            }

            match Vehicle::reserve(path_grid, self.id, *pos, &mut reserved) {
                ReservationStatus::TileSuccess => {
                    match self.should_we_yield_when_entering(path_grid, &my_tile, &pos) {
                        ShouldWeYieldStatus::Yield(yield_to_pos) => {
                            Vehicle::clear_reserved(path_grid, &mut reserved);
                            return ReservePathStatus::Blocking(yield_to_pos);
                        }
                        ShouldWeYieldStatus::Clear => return ReservePathStatus::Success(reserved),
                    }
                }
                ReservationStatus::TileInvalid => {
                    return ReservePathStatus::InvalidPath;
                }
                ReservationStatus::TileReserved => {
                    return ReservePathStatus::Blocking(*pos);
                }
            }
        }

        ReservePathStatus::Success(reserved)
    }

    fn update_speed(&mut self) {
        self.lag_pos_pixels -= SPEED_PIXELS_PER_TICK;
    }

    fn find_path(&mut self, grid: &mut Grid) -> bool {
        self.path = grid.find_path(&self.pos, &self.destination);

        if let Some(path) = &self.path {
            self.path_time_ticks = path.1 * SPEED_TICKS_PER_TILE as u32;
            self.path_index = 0;
        }
        self.path.is_some()
    }

    fn get_next_pos(&mut self, grid: &mut Grid) -> Option<Position> {
        match self.reserve_path(grid) {
            ReservePathStatus::Success(reserved) => {
                self.reserved = Some(reserved);
                if let Some(reserved) = &self.reserved {
                    self.path_index += 1;
                    return Some(reserved[0]);
                } else {
                    return None;
                }
            }
            ReservePathStatus::InvalidPath => {
                self.find_path(grid);
                grid.reserve_position(&self.pos, self.id);
                return None;
            }
            ReservePathStatus::Blocking(blocking_pos) => {
                self.blocking_tile = Some(blocking_pos);
                grid.reserve_position(&self.pos, self.id);
                return None;
            }
        }
    }

    fn update_position(&mut self, path_grid: &mut Grid) -> Status {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Tile::Road(road) = path_grid.get_tile(&blocking_tile) {
                if road.reserved.is_some() {
                    // don't bother
                    return Status::EnRoute;
                }
            }
        }
        self.blocking_tile = None;

        if self.pos == self.destination {
            return Status::ReachedDestination;
        }

        if let Some(reserved) = &mut self.reserved {
            Vehicle::clear_reserved(path_grid, reserved);
            self.reserved = None;
        }

        path_grid.unreserve_position(&self.pos);

        if let Some(next_pos) = self.get_next_pos(path_grid) {
            self.lag_pos_pixels = (GRID_CELL_SIZE.0 as i16) - SPEED_PIXELS_PER_TICK;
            self.dir = self.pos.direction_to(next_pos);
            self.pos = next_pos;
        }

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.path {
            let tiles_elapsed = (self.elapsed_ticks.saturating_sub(1) / SPEED_TICKS_PER_TILE as u32) + 1;
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
        } else if self.lag_pos_pixels > 0 {
            self.update_speed();
            Status::EnRoute
        } else {
            self.update_position(path_grid)
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        let mut rect = Rect::from(self.pos);
        match self.dir {
            Direction::Right => rect.x -= self.lag_pos_pixels as f32,
            Direction::Left => rect.x += self.lag_pos_pixels as f32,
            Direction::Down => rect.y -= self.lag_pos_pixels as f32,
            Direction::Up => rect.y += self.lag_pos_pixels as f32,
        }

        let vehicle_red = Color::from_hex(0xf9524c);
        let vehicle_blue = Color::from_hex(0xa0dae8);
        let vehicle_yellow = Color::from_hex(0xf8c768);

        let mut color = vehicle_blue;

        if self.path.is_none() {
            color = vehicle_red;
        // } else if self.blocking_tile.is_some() {
        } else if self.trip_late() < 0.75 {
            color = vehicle_yellow;
        }

        let sprite = 1;

        // draw shadow
        let mut shadow_rect = rect;
        shadow_rect.x += 2.;
        shadow_rect.y += 2.;
        tileset.draw_tile(2, WHITE, &shadow_rect, self.dir.to_radians());

        tileset.draw_tile(sprite, color, &rect, self.dir.to_radians());
    }

    pub(crate) fn draw_detail(&self, tileset: &Tileset) {
        // draw reserved
        let mut reserved_path_color = RED;
        reserved_path_color.a = 0.3;
        if let Some(reserved) = &self.reserved {
            for pos in reserved {
                tileset.draw_rect(&Rect::from(*pos), reserved_path_color);
            }
        }

        let mut path_color = BLUE;
        path_color.a = 0.3;
        if let Some(path) = self.path.as_ref() {
            for pos in &path.0 {
                tileset.draw_rect(&Rect::from(*pos), path_color);
            }
        }
    }
}

#[cfg(test)]
mod vehicle_tests {

    use crate::grid::ReservationStatus;

    use super::*;

    #[test]
    fn test_init() {
        let mut grid = Grid::new_from_string(">>>>");
        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(3, 0);
        assert!(Vehicle::new(start_pos, 0, end_pos, &mut grid).is_some());

        assert_eq!(
            grid.reserve_position(&start_pos, 1),
            ReservationStatus::TileReserved
        );

        assert!(Vehicle::new(start_pos, 2, end_pos, &mut grid).is_none());
    }

    #[test]
    fn test_status() {
        let mut grid = Grid::new_from_string(">>>>");
        let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, grid.pos(3, 0), &mut grid).unwrap();

        grid.reserve_position(&grid.pos(1, 0), 1);

        assert_eq!(vehicle.update(&mut grid), Status::EnRoute);
    }

    #[test]
    fn test_late() {
        let mut grid = Grid::new_from_string(">>>>");
        let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, grid.pos(3, 0).into(), &mut grid).unwrap();

        vehicle.update(&mut grid);

        vehicle.elapsed_ticks = SPEED_TICKS_PER_TILE as u32 + 1;
        assert_eq!(vehicle.trip_late(), 0.6666666);

        vehicle.elapsed_ticks = (SPEED_TICKS_PER_TILE * 2) as u32 + 1;
        assert_eq!(vehicle.trip_late(), 0.33333337);
    }

    #[test]
    fn test_trip() {
        let mut grid = Grid::new_from_string(">>>>");
        let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, grid.pos(3, 0), &mut grid).unwrap();

        let trip_length: u32 = 3;
        let trip_time = SPEED_TICKS_PER_TILE as u32 * (trip_length - 0);

        assert_eq!(vehicle.path_time_ticks, trip_time);

        assert_eq!(vehicle.trip_completed_percent(), 0.);

        assert_eq!(vehicle.trip_late(), 1.0);
        // grid.reserve_position(&(1, 0).into(), 1);

        // assert_eq!(vehicle.update(&mut grid), Status::EnRoute);

        for i in 0..24 {
            assert_eq!(
                vehicle.update(&mut grid),
                Status::EnRoute,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.path_index,
                1 + (i / (SPEED_TICKS_PER_TILE as u32)) as usize,
                "Failed on tick {i}"
            );
            assert_eq!(vehicle.elapsed_ticks, i + 1);
            assert_eq!(
                vehicle.trip_completed_percent(),
                ((i + 8) / SPEED_TICKS_PER_TILE as u32) as f32 / trip_length as f32,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.trip_late(),
                1.0,
                "Failed on tick {i} %{}",
                vehicle.trip_completed_percent()
            );
        }

        assert_eq!(
            vehicle.update(&mut grid),
            Status::ReachedDestination
        );
        assert_ne!(vehicle.trip_late(), 1.0);
    }

    #[test]
    fn test_reserved() {
        let mut grid = Grid::new_from_string(">>>>");

        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(3, 0);

        let _ = Vehicle::new(start_pos, 0, end_pos, &mut grid).unwrap();

        assert_eq!(
            grid.reserve_position(&end_pos, 1),
            ReservationStatus::TileSuccess
        );

        assert_eq!(
            grid.reserve_position(&end_pos, 1),
            ReservationStatus::TileReserved
        );
    }

    #[test]
    fn test_update_speed() {
        let mut grid = Grid::new_from_string(">>>>");
        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(3, 0);
        let mut vehicle = Vehicle::new(start_pos, 0, end_pos, &mut grid).unwrap();

        vehicle.update_speed();

        assert_eq!(vehicle.lag_pos_pixels, -SPEED_PIXELS_PER_TICK);
    }

    #[test]
    fn test_blocking_tile() {}

    #[test]
    fn test_yield() {
        let mut grid = Grid::new_from_string(
            "\
            >>>>>
            _h___",
        );

        // println!("grid: {:?}", &grid);

        let start_pos = grid.pos(1, 1);
        let yield_to_pos = grid.pos(0, 0);
        let intersection_pos = grid.pos(1, 0);
        let mut vehicle = Vehicle::new(start_pos, 0, grid.pos(3, 0), &mut grid).unwrap();

        assert_eq!(
            grid.reserve_position(&yield_to_pos, 1),
            ReservationStatus::TileSuccess
        );

        vehicle.update(&mut grid);

        assert_eq!(
            grid.reserve_position(&intersection_pos, 1),
            ReservationStatus::TileSuccess
        );
    }

    #[test]
    fn test_yield_roundabout() {
        let mut grid = Grid::new_from_string(
            "\
            __.^__
            __.^__
            <<lr<<
            >>LR>>
            __.^__
            __.^__
            ",
        );

        let mut vehicle_top =
            Vehicle::new(grid.pos(2, 1), 0, grid.pos(2, 4), &mut grid).unwrap();

        let mut vehicle_left =
            Vehicle::new(grid.pos(1, 3), 1, grid.pos(5, 3), &mut grid).unwrap();

        let mut vehicle_bottom =
            Vehicle::new(grid.pos(3, 4), 2, grid.pos(3, 0), &mut grid).unwrap();

        let mut vehicle_right =
            Vehicle::new(grid.pos(4, 2), 3, grid.pos(0, 2), &mut grid).unwrap();

        assert!(vehicle_top.path.is_some());
        assert!(vehicle_left.path.is_some());
        assert!(vehicle_bottom.path.is_some());
        assert!(vehicle_right.path.is_some());

        println!("grid: \n{:?}", grid);

        vehicle_top.update(&mut grid);
        vehicle_left.update(&mut grid);
        vehicle_bottom.update(&mut grid);
        vehicle_right.update(&mut grid);

        println!("grid after: \n{:?}", grid);

        assert!(vehicle_top.reserved.is_some());
        assert!(vehicle_left.reserved.is_none());
        assert!(vehicle_bottom.reserved.is_some());
        assert!(vehicle_right.reserved.is_none());
    }

    #[test]
    fn test_yield_house() {
        // Houses should yield, but only to relevant traffic
        let mut grid = Grid::new_from_string(
            "\
            <<<<
            >>>>
            _h__",
        );

        let mut vehicle =
            Vehicle::new(grid.pos(1, 2), 0, grid.pos(3, 1), &mut grid).unwrap();

        let yield_to_pos = grid.pos(0, 1);

        assert_eq!(vehicle.path.is_some(), true);

        // reserve position we should yield to
        grid.reserve_position(&yield_to_pos, 1);

        vehicle.update(&mut grid);

        assert_eq!(vehicle.blocking_tile.unwrap(), yield_to_pos);

        grid.unreserve_position(&yield_to_pos);

        // reserve position accross the street
        let do_not_yield_to_pos = grid.pos(1, 0);
        grid.reserve_position(&do_not_yield_to_pos, 1);

        vehicle.update(&mut grid);

        // TODO!!
        // assert!(vehicle.path_status, PathStatus::Okay);
    }
}
