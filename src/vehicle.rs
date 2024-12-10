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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    yield_delay: bool,
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
            yield_delay: false,
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

    fn reserve_path(&mut self, path_grid: &mut Grid, positions: &Vec<Position>) -> bool {
        for pos in positions {
            if *pos == self.pos {
                continue;
            }

            match self.reserve(path_grid, *pos) {
                ReservationStatus::TileBlockable => {
                    return true;
                }
                ReservationStatus::TileDoNotBlock => {
                    // keep reserving ahead
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

    fn update_path(&mut self, path_grid: &mut Grid) -> Option<Position> {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
                if road.reserved {
                    // don't bother
                    self.yield_delay = false;
                    return None;
                }
                if path_grid.should_yield(&self.pos) {
                    if self.yield_delay {
                        self.yield_delay = false;
                    } else {
                        self.yield_delay = true;
                        return None;
                    }
                }
            }
        }
        self.blocking_tile = None;
        self.yield_delay = false;

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
            self.update_path(path_grid)
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

        // draw the path
        // if let Some(path) = &self.path {
        //     for seg in &path.0 {
        //     }
        // }
    }
}

#[cfg(test)]
mod vehicle_tests {
    use crate::grid::{House, ReservationStatus};

    use super::*;

    fn new_grid_from_ascii(ascii: &str) -> Grid {
        let mut pos = Position::new(0, 0);
        let mut grid = Grid::new(ascii.len(), 1);
        for chr in ascii.chars() {
            match chr {
                '>' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                }
                '*' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                    grid.add_tile_connection(&pos, Direction::Up);
                }
                'y' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                    if let Tile::Road(road) = grid.get_tile_mut(&pos).unwrap() {
                        road.should_yield = true;
                    }
                }
                'h' => {
                    *grid.get_tile_mut(&pos).unwrap() = Tile::House(House {
                        people_heading_to: true,
                    });
                }
                _ => {}
            }
            pos.x += 1;
        }

        grid
    }

    #[test]
    fn test_init() {
        let mut grid = new_grid_from_ascii(">*>>>>>>");
        let start_pos = Position::new(0, 0);
        let end_pos = Position::new(4, 0);
        let mut vehicle = Vehicle::new(start_pos, end_pos, &mut grid).unwrap();
        assert_eq!(
            grid.reserve_position(&start_pos),
            ReservationStatus::TileReserved
        );

        vehicle.update(&mut grid);

        assert!(vehicle.path_status == PathStatus::Okay);

        assert_eq!(
            grid.reserve_position(&start_pos),
            ReservationStatus::TileBlockable
        );
    }

    #[test]
    fn test_yield() {
        let mut grid = new_grid_from_ascii("h>>>>");
        let start_pos = Position::new(0, 0);
        let yield_to_pos = Position::new(1, 0);
        let mut vehicle = Vehicle::new(start_pos, Position::new(3, 0), &mut grid).unwrap();

        assert_eq!(
            grid.reserve_position(&yield_to_pos),
            ReservationStatus::TileBlockable
        );

        vehicle.update(&mut grid);

        grid.unreserve_position(&yield_to_pos);

        vehicle.update(&mut grid);

        // we should yield and not move immediantly
        assert_eq!(
            grid.reserve_position(&yield_to_pos),
            ReservationStatus::TileBlockable
        );

        vehicle.update(&mut grid);

        grid.unreserve_position(&yield_to_pos);

        vehicle.update(&mut grid);

        // even if it happend again
        assert_eq!(
            grid.reserve_position(&yield_to_pos),
            ReservationStatus::TileBlockable
        );
    }
}
