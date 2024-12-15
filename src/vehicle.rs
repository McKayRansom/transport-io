use macroquad::color::Color;
use macroquad::color::WHITE;
use macroquad::math::Rect;

use crate::grid::Direction;
use crate::grid::Grid;
use crate::grid::Path;
use crate::grid::Position;
use crate::grid::ReservationStatus;
use crate::grid::Tile;
use crate::grid::GRID_CELL_SIZE;
use crate::tileset::Tileset;

const SPEED: i16 = 4;
const HOPELESSLY_LATE_PERCENT: f32 = 0.25;

pub struct Vehicle {
    path: Path,
    eta: u32,
    elapsed: u32,
    pos: Position,
    lag_pos: i16,
    dir: Direction,
    // station_id: usize
    blocking_tile: Option<Position>,
    destination: Position,
    reserved: Option<Vec<Position>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd)]
pub enum Status {
    EnRoute,
    ReachedDestination(Position),
    HopelesslyLate(Position),
}

pub enum ReservePathStatus {
    InvalidPath,
    Success(Vec<Position>),
    Blocking(Position),
}

pub enum ShouldWeYieldStatus {
    Yield(Position),
    Clear
}

impl Vehicle {
    pub fn new(pos: Position, destination: Position, grid: &mut Grid) -> Option<Self> {
        if grid.reserve_position(&pos) != ReservationStatus::TileBlockable {
            return None;
        }

        // get path now?

        let mut vehicle = Vehicle {
            path: None,
            eta: 0,
            elapsed: 0,
            pos: pos,
            lag_pos: 0,
            dir: Direction::Right,
            blocking_tile: None,
            destination: destination,
            reserved: Some(vec![pos]),
        };

        vehicle.find_path(grid);

        if let Some(path) = &vehicle.path {
            vehicle.eta = path.1 * SPEED as u32;
        }

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
        position: Position,
        reserved: &mut Vec<Position>,
    ) -> ReservationStatus {
        let status = path_grid.reserve_position(&position);
        if status == ReservationStatus::TileBlockable {
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
            if road.connections.count() > 1 {
                return ShouldWeYieldStatus::Clear;
            }
        }

        if let Some(Tile::Road(road)) = path_grid.get_tile(position) {
            // For each direction that feeds into this tile in question
            for dir in road
                .connections
                .iter_inverse(crate::grid::ConnectionLayer::Road)
            {
                let yield_to_pos = Position::new_from_move(position, dir);

                // Get the road
                if let Some(Tile::Road(yield_to_road)) = path_grid.get_tile(&yield_to_pos) {
                    // if it's reserved and connects to our road
                    if yield_to_road.reserved && yield_to_road.connections.has(dir.inverse()) {
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
                                if yield_to_road.connections.count() > 1 {
                                    return ShouldWeYieldStatus::Yield(yield_to_pos);
                                }
                                // otherwise if the tile is a normal road we probably don't need to yield
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
        let my_tile = *path_grid.get_tile(&self.pos).unwrap();
        let mut reserved = Vec::<Position>::new();
        // println!("should_yield: {}", should_yield);
        if self.path.is_none() {
            return ReservePathStatus::InvalidPath
        }
        for pos in &self.path.as_ref().unwrap().0 {
            if *pos == self.pos {
                continue;
            }

            match Vehicle::reserve(path_grid, *pos, &mut reserved) {
                ReservationStatus::TileBlockable => {
                    match self.should_we_yield_when_entering(path_grid, &my_tile, &pos) {
                        ShouldWeYieldStatus::Yield(yield_to_pos) => {
                            Vehicle::clear_reserved(path_grid, &mut reserved);
                            return ReservePathStatus::Blocking(yield_to_pos);
                        }
                        ShouldWeYieldStatus::Clear => {
                            return ReservePathStatus::Success(reserved)
                        }
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
        self.lag_pos -= SPEED;
    }

    fn find_path(&mut self, grid: &mut Grid) -> bool {
        self.path = grid.find_path(&self.pos, &self.destination);
        self.path.is_some()
    }

    fn get_next_pos(&mut self, grid: &mut Grid) -> Option<Position> {

        match self.reserve_path(grid) {
            ReservePathStatus::Success(reserved) => {
                self.reserved = Some(reserved);
                if let Some(reserved) = &self.reserved {
                    if let Some(path) = &mut self.path {
                        // Not ideal performance
                        path.0.remove(0);
                    }
                    return Some(reserved[0]);
                } else {
                    return None;
                }
            }
            ReservePathStatus::InvalidPath => {
                self.find_path(grid);
                grid.reserve_position(&self.pos);
                return None;
            }
            ReservePathStatus::Blocking(blocking_pos) => {
                self.blocking_tile = Some(blocking_pos);
                grid.reserve_position(&self.pos);
                return None;
            }
        }
    }

    fn update_position(&mut self, path_grid: &mut Grid) -> Status {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
                if road.reserved {
                    // don't bother
                    return Status::EnRoute;
                }
            }
        }
        self.blocking_tile = None;

        if self.pos == self.destination {
            return Status::ReachedDestination(self.destination);
        }

        if let Some(reserved) = &mut self.reserved {
            Vehicle::clear_reserved(path_grid, reserved);
            self.reserved = None;
        }

        path_grid.unreserve_position(&self.pos);

        if let Some(next_pos) = self.get_next_pos(path_grid) {
            self.lag_pos = (GRID_CELL_SIZE.0 as i16) - SPEED;
            self.dir = Direction::from_position(self.pos, next_pos);
            self.pos = next_pos;
        }

        Status::EnRoute
    }

    pub fn on_time_percent(&self) -> f32 {
        // 1 = on time exactly
        // 0.5 = 50% more time taken
        (self.eta as f32 / self.elapsed as f32).min(1.0)
    }

    pub fn update(&mut self, path_grid: &mut Grid) -> Status {
        self.elapsed += 1;
        if self.on_time_percent() < HOPELESSLY_LATE_PERCENT {
            Status::HopelesslyLate(self.destination)
        } else if self.lag_pos > 0 {
            self.update_speed();
            Status::EnRoute
        } else {
            self.update_position(path_grid)
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        let mut rect = Rect::from(self.pos);
        match self.dir {
            Direction::Right => rect.x -= self.lag_pos as f32,
            Direction::Left => rect.x += self.lag_pos as f32,
            Direction::Down => rect.y -= self.lag_pos as f32,
            Direction::Up => rect.y += self.lag_pos as f32,
        }

        let vehicle_red = Color::from_hex(0xf9524c);
        let vehicle_blue = Color::from_hex(0xa0dae8);
        let vehicle_yellow = Color::from_hex(0xf8c768);

        let mut color = vehicle_blue;

        if self.path.is_none() {
            color = vehicle_red;
        } else if self.blocking_tile.is_some() {
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
}

#[cfg(test)]
mod vehicle_tests {

    use crate::grid::ReservationStatus;

    use super::*;

    #[test]
    fn test_init() {
        let mut grid = Grid::new_grid_from_ascii(">>>>");
        let start_pos = Position::new(0, 0);
        let end_pos = Position::new(3, 0);
        assert!(Vehicle::new(start_pos, end_pos, &mut grid).is_some());

        assert_eq!(
            grid.reserve_position(&start_pos),
            ReservationStatus::TileReserved
        );

        assert!(Vehicle::new(start_pos, end_pos, &mut grid).is_none());
    }

    #[test]
    fn test_status() {
        let mut grid = Grid::new_grid_from_ascii(">>>>");
        let mut vehicle = Vehicle::new((0, 0).into(), (3, 0).into(), &mut grid).unwrap();

        grid.reserve_position(&(1, 0).into());

        assert_eq!(vehicle.update(&mut grid), Status::EnRoute);
    }

    #[test]
    fn test_late() {
            
        let mut grid = Grid::new_grid_from_ascii(">>>>");
        let mut vehicle = Vehicle::new((0, 0).into(), (3, 0).into(), &mut grid).unwrap();

        assert_eq!(vehicle.eta, SPEED as u32 * 6);

        assert_eq!(vehicle.on_time_percent(), 1.0);
        grid.reserve_position(&(1, 0).into());

        for i in 1..25 {
            assert_eq!(vehicle.update(&mut grid), Status::EnRoute);
            assert_eq!(vehicle.elapsed, i);
            assert_eq!(vehicle.on_time_percent(), 1.0);
        }

        assert_eq!(vehicle.update(&mut grid), Status::EnRoute);
        assert_ne!(vehicle.on_time_percent(), 1.0);
    }

    #[test]
    fn test_reserved() {
        let mut grid = Grid::new_grid_from_ascii(">>>>");

        let start_pos = (0, 0).into();
        let end_pos = (3, 0).into();

        let _ = Vehicle::new(start_pos, end_pos, &mut grid).unwrap();

        assert_eq!(
            grid.reserve_position(&end_pos),
            ReservationStatus::TileBlockable
        );

        assert_eq!(
            grid.reserve_position(&end_pos),
            ReservationStatus::TileReserved
        );
    }

    #[test]
    fn test_update_speed() {
        let mut grid = Grid::new_grid_from_ascii(">>>>");
        let start_pos = Position::new(0, 0);
        let end_pos = Position::new(3, 0);
        let mut vehicle = Vehicle::new(start_pos, end_pos, &mut grid).unwrap();

        vehicle.update_speed();

        assert_eq!(vehicle.lag_pos, -SPEED);
    }

    #[test]
    fn test_blocking_tile() {}

    #[test]
    fn test_yield() {
        let mut grid = Grid::new_grid_from_ascii(
            "\
            >>>>>
            _h___",
        );

        // println!("grid: {:?}", &grid);

        let start_pos = Position::new(1, 1);
        let yield_to_pos = Position::new(0, 0);
        let intersection_pos = Position::new(1, 0);
        let mut vehicle = Vehicle::new(start_pos, Position::new(3, 0), &mut grid).unwrap();

        assert_eq!(
            grid.reserve_position(&yield_to_pos),
            ReservationStatus::TileBlockable
        );

        vehicle.update(&mut grid);

        assert_eq!(
            grid.reserve_position(&intersection_pos),
            ReservationStatus::TileBlockable
        );
    }

    #[test]
    fn test_yield_roundabout() {
        let mut grid = Grid::new_grid_from_ascii(
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
            Vehicle::new(Position::new(2, 1), Position::new(2, 4), &mut grid).unwrap();

        let mut vehicle_left =
            Vehicle::new(Position::new(1, 3), Position::new(5, 3), &mut grid).unwrap();

        let mut vehicle_bottom =
            Vehicle::new(Position::new(3, 4), Position::new(3, 0), &mut grid).unwrap();

        let mut vehicle_right =
            Vehicle::new(Position::new(4, 2), Position::new(0, 2), &mut grid).unwrap();

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
        let mut grid = Grid::new_grid_from_ascii(
            "\
            <<<<
            >>>>
            _h__",
        );

        let mut vehicle =
            Vehicle::new(Position::new(1, 2), Position::new(3, 1), &mut grid).unwrap();

        let yield_to_pos = Position::new(0, 1);

        assert_eq!(vehicle.path.is_some(), true);

        // reserve position we should yield to
        grid.reserve_position(&yield_to_pos);

        vehicle.update(&mut grid);

        assert_eq!(vehicle.blocking_tile.unwrap(), yield_to_pos);

        grid.unreserve_position(&yield_to_pos);

        // reserve position accross the street
        let do_not_yield_to_pos = Position::new(1, 0);
        grid.reserve_position(&do_not_yield_to_pos);

        vehicle.update(&mut grid);

        // TODO!!
        // assert!(vehicle.path_status, PathStatus::Okay);
    }
}
