use macroquad::color::Color;
use macroquad::color::WHITE;
use macroquad::math::Rect;

use crate::grid::Direction;
use crate::grid::Grid;
use crate::grid::Position;
use crate::grid::ReservationStatus;
use crate::grid::Tile;
use crate::grid::GRID_CELL_SIZE;
use crate::tileset::Tileset;

const SPEED: i16 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PathStatus {
    Okay,
    Waiting,
    NoPath,
}

pub struct Vehicle {
    pos: Position,
    lag_pos: i16,
    dir: Direction,
    // station_id: usize
    blocking_tile: Option<Position>,
    destination: Position,
    reserved: Vec<Position>,
    path_status: PathStatus,
}

impl Vehicle {
    pub fn new(pos: Position, destination: Position, path_grid: &mut Grid) -> Option<Self> {
        if path_grid.reserve_position(&pos) != ReservationStatus::TileBlockable {
            return None;
        }

        Some(Vehicle {
            pos: pos,
            lag_pos: 0,
            dir: Direction::Right,
            blocking_tile: None,
            destination: destination,
            reserved: vec![pos],
            path_status: PathStatus::Okay,
        })
    }

    fn reserve(&mut self, path_grid: &mut Grid, position: Position) -> ReservationStatus {
        let status = path_grid.reserve_position(&position);
        if status == ReservationStatus::TileBlockable || status == ReservationStatus::TileDoNotBlock
        {
            self.reserved.push(position);
        } else {
            self.clear_reserved(path_grid);
        }
        status
    }

    pub fn clear_reserved(&mut self, path_grid: &mut Grid) {
        for pos in &self.reserved {
            path_grid.unreserve_position(pos);
        }
        self.reserved.clear();
    }

    fn should_we_yield_when_entering(
        &mut self,
        path_grid: &Grid,
        my_tile: &Tile,
        position: &Position,
    ) -> bool {
        // For now: Assume we are right before the intersection
        // TODO: New yield logic:
        // 1. Always yield when entering an intersection to other itersection tiles that feed to here
        // 2. Always yield when exiting a house (for now)
        // 3. Always yield if we have a yield sign
        // let mut my_tile_is_intersection = false;
        if let Tile::Road(road) = my_tile {
            if road.connections.count() > 1 {
                // don't yield from an intersection
                return false;
                // my_tile_is_intersection = true;
            }
        }

        // println!("Checking neighbors of: {:?}", position);

        if let Some(Tile::Road(road)) = path_grid.get_tile(position) {
            // For each direction that feeds into this tile in question
            for dir in road.connections.iter_inverse(crate::grid::ConnectionLayer::Road) {
                let yield_to_pos = Position::new_from_move(position, dir);

                // Get the road
                if let Some(Tile::Road(yield_to_road)) = path_grid.get_tile(&yield_to_pos) {

                    // if it's reserved and connects to our road
                    if yield_to_road.reserved && yield_to_road.connections.has(dir.inverse()) {
                        match my_tile {
                            // alway yield from a house
                            Tile::House(_) => {
                                return true;
                            }
                            // if we are somehow in a weird state, I guess yield?
                            Tile::Empty => {
                                return true;
                            }
                            Tile::Road(_) => {
                                // Always yield when entering an interseciton (if we aren't an intersection)
                                // because of the check above we know we are NOT an intersection
                                if yield_to_road.connections.count() > 1 {
                                    // don't bother
                                    self.blocking_tile = Some(yield_to_pos);
                                    // println!("hi");
                                    return true;
                                }
                                else {
                                    // println!("hello from {:?} we are: {:?}", yield_to_pos, yield_to_road);
                                    // keep checking
                                }
                            }
                        }
                    }
                }
            }
        }

        return false;
    }

    fn reserve_path(&mut self, path_grid: &mut Grid, positions: &Vec<Position>) -> bool {
        // let mut should_yield = path_grid.should_yield(&self.pos);
        let my_tile = *path_grid.get_tile(&self.pos).unwrap();
        // println!("should_yield: {}", should_yield);
        for pos in positions {
            if *pos == self.pos {
                continue;
            }

            match self.reserve(path_grid, *pos) {
                ReservationStatus::TileBlockable => {
                    if self.should_we_yield_when_entering(path_grid, &my_tile, pos) {
                        return false;
                    }
                    return true;
                }
                ReservationStatus::TileDoNotBlock => {
                    // if should_yield {
                    if self.should_we_yield_when_entering(path_grid, &my_tile, pos) {
                        return false;
                    } else {
                        return true;
                    }
                    // }
                    // else {
                    // return true;
                    // }
                    // keep reserving ahead
                    // should_yield = false;
                    // continue;
                }
                ReservationStatus::TileInvalid => {
                    // TODO: Return calc new path?
                    return false;
                }
                ReservationStatus::TileReserved => {
                    // TODO: Only do this if we are not currently in an intersection
                    self.blocking_tile = Some(*pos);
                    return false;
                }
            }
        }

        true
    }

    fn update_speed(&mut self) {
        self.lag_pos -= SPEED;
    }

    fn get_next_pos(&mut self, path_grid: &mut Grid) -> Option<Position> {
        if let Some(path) = path_grid.find_path(&self.pos, &self.destination) {
            if !self.reserve_path(path_grid, &path.0) {
                self.clear_reserved(path_grid);
                self.path_status = PathStatus::Waiting;
                return None;
            }

            self.path_status = PathStatus::Okay;

            return path.0.get(1).copied();
        } else {
            self.path_status = PathStatus::NoPath;
            return None;
        }
    }

    fn update_position(&mut self, path_grid: &mut Grid) -> Option<Position> {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
                if road.reserved {
                    // don't bother
                    return None;
                }
            }
        }
        self.blocking_tile = None;

        if self.pos == self.destination {
            return Some(self.destination);
        }

        self.clear_reserved(path_grid);

        if let Some(next_pos) = self.get_next_pos(path_grid) {
            self.lag_pos = (GRID_CELL_SIZE.0 as i16) - SPEED;
            self.dir = Direction::from_position(self.pos, next_pos);
            self.pos = next_pos;
        } else {
            self.reserve(path_grid, self.pos);
        }

        None
    }

    pub fn update(&mut self, path_grid: &mut Grid) -> Option<Position> {
        if self.lag_pos > 0 {
            self.update_speed();
            None
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

        let color = match self.path_status {
            PathStatus::NoPath => Color::from_hex(0xf9524c),
            PathStatus::Okay => Color::from_hex(0xa0dae8),
            PathStatus::Waiting => Color::from_hex(0xf8c768),
        };

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
    fn test_reserved() {
        let mut grid = Grid::new_grid_from_ascii(">>>>");
        let start_pos = Position::new(0, 0);
        let end_pos = Position::new(3, 0);
        let mut vehicle = Vehicle::new(start_pos, end_pos, &mut grid).unwrap();

        assert_eq!(
            vehicle.reserve(&mut grid, end_pos),
            ReservationStatus::TileBlockable
        );

        assert_eq!(
            vehicle.reserve(&mut grid, end_pos),
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

        assert_eq!(vehicle_top.path_status, PathStatus::Okay);
        assert_eq!(vehicle_left.path_status, PathStatus::Okay);
        assert_eq!(vehicle_bottom.path_status, PathStatus::Okay);
        assert_eq!(vehicle_right.path_status, PathStatus::Okay);

        println!("grid: \n{:?}", grid);

        vehicle_top.update(&mut grid);
        vehicle_left.update(&mut grid);
        vehicle_bottom.update(&mut grid);
        vehicle_right.update(&mut grid);

        println!("grid after: \n{:?}", grid);

        assert_eq!(vehicle_top.path_status, PathStatus::Okay);
        assert_eq!(vehicle_left.path_status, PathStatus::Waiting);
        assert_eq!(vehicle_bottom.path_status, PathStatus::Okay);
        assert_eq!(vehicle_right.path_status, PathStatus::Waiting);

    }

    #[test]
    fn test_yield_house() {
        // Houses should yield, but only to relevant traffic
        let mut grid = Grid::new_grid_from_ascii(
            "\
            <<<<
            >>>>
            _h__"
        );


        let mut vehicle =
            Vehicle::new(Position::new(1, 2), Position::new(3, 1), &mut grid).unwrap();

        let yield_to_pos = Position::new(0, 1);

        assert_eq!(vehicle.path_status, PathStatus::Okay);

        // reserve position we should yield to
        grid.reserve_position(&yield_to_pos);

        vehicle.update(&mut grid);

        assert_eq!(vehicle.path_status, PathStatus::Waiting);

        grid.unreserve_position(&yield_to_pos);

        // reserve position accross the street
        let do_not_yield_to_pos = Position::new(1, 0);
        grid.reserve_position(&do_not_yield_to_pos);

        vehicle.update(&mut grid);
 
        assert_eq!(vehicle.path_status, PathStatus::Okay);

    }
}
