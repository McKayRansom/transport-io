use macroquad::color::Color;
use macroquad::color::WHITE;

use crate::grid::Connections;
use crate::grid::Direction;
use crate::grid::Grid;
use crate::grid::Position;
use crate::grid::Rectangle;
use crate::grid::Reservation;
use crate::grid::ReservationInfo;
use crate::grid::Tile;
use crate::grid::GRID_CELL_SIZE;
// use crate::station;
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
    // station_id: usize,
    blocking_tile: Option<Position>,
    destination: Position,
    reserved: Vec<Reservation>,
    path_status: PathStatus,
    yield_delay: bool,
}

impl Vehicle {
    pub fn new(pos: Position, destination: Position, path_grid: &mut Grid) -> Option<Self> {
        let res = Reservation {
            position: pos,
            start_tick: 0,
            end_tick: 31,
        };
        if let Some((connection, info)) = path_grid.reserve_position(&res) {
            if !connection.safe_to_block() && !info.later_reservation {
                return None;
            }
        } else {
            return None;
        }

        Some(Vehicle {
            pos: pos,
            lag_pos: 0,
            dir: Direction::Right,
            // station_id: 0,
            blocking_tile: None,
            destination: destination,
            reserved: vec![res], // TODO: Safe way to do this?
            path_status: PathStatus::Okay,
            yield_delay: false,
        })
    }

    fn reserve(
        &mut self,
        path_grid: &mut Grid,
        reservation: &Reservation,
    ) -> Option<(Connections, ReservationInfo)> {
        if let Some(info) = path_grid.reserve_position(reservation) {
            self.reserved.push(*reservation);
            Some(info)
        } else {
            None
        }
    }

    pub fn clear_reserved(&mut self, path_grid: &mut Grid) {
        // shouldn't double-free
        // assert!(self.reserved.len() > 0);

        for pos in &self.reserved {
            path_grid.unreserve_position(pos);
        }
        self.reserved.clear();
    }

    fn reserve_path(&mut self, path_grid: &mut Grid, positions: &Vec<Position>) -> bool {
        let mut reservation = Reservation {
            position: self.pos,
            start_tick: 0,
            end_tick: SPEED as u32,
        };

        for pos in positions {
            if *pos == self.pos {
                continue;
            }

            reservation.position = *pos;

            if let Some(connection) = self.reserve(path_grid, &reservation) {
                // find safe place to wait
                if connection.0.safe_to_block() {
                    // check for delay TODO: Change to generic check for tile that should yield
                    if path_grid.should_yield(&self.pos) {
                        if self.yield_delay {
                            self.yield_delay = false;
                            break;
                        }
                        self.yield_delay = true;
                        self.clear_reserved(path_grid);
                        return false;
                    }
                    break;
                }
            } else {
                self.blocking_tile = Some(*pos);
                self.yield_delay = false;
                self.clear_reserved(path_grid);
                return false;
            }
        }

        true
    }

    fn update_position(&mut self) {
        self.lag_pos -= SPEED;
    }

    fn update_path(&mut self, path_grid: &mut Grid) -> Option<Position> {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
                if road.reservations.is_reserved(0, 4) {
                    // don't bother
                    return None;
                }
            }
        }
        self.blocking_tile = None;

        // check destination
        if self.pos == self.destination {
            //stations[self.station_id].pos {
            // we made it! head to next station
            // self.station_id += 1;
            // if self.station_id >= stations.len() {
            //     self.station_id = 0;
            // }
            return Some(self.destination);
        }

        self.clear_reserved(path_grid);
        // let destination = &stations[self.station_id];

        if let Some(path) = path_grid.find_path(&self.pos, &self.destination) {
            if !self.reserve_path(path_grid, &path.0) {
                self.reserve(
                    path_grid,
                    &Reservation {
                        position: self.pos,
                        start_tick: 0,
                        end_tick: 31,
                    },
                );

                self.path_status = PathStatus::Waiting;
                return None;
            }

            self.path_status = PathStatus::Okay;

            let positions = &path.0;
            if positions.len() == 0 {
                return None;
            }

            let next_pos = positions[1];
            self.lag_pos = (GRID_CELL_SIZE.0 as i16) - SPEED;
            self.dir = Direction::from_position(self.pos, next_pos);
            self.pos = next_pos;
        } else {
            self.path_status = PathStatus::NoPath;
            self.reserve(
                path_grid,
                &Reservation {
                    position: self.pos,
                    start_tick: 0,
                    end_tick: 31,
                },
            );
        }

        None
    }

    pub fn update(&mut self, path_grid: &mut Grid) -> Option<Position> {
        if self.lag_pos > 0 {
            self.update_position();
            None
        } else {
            self.update_path(path_grid)
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        // let mut width_fraction: f32 = 0.9;
        // let mut height_fraction: f32 = 0.9;
        // if self.dir == Direction::Right || self.dir == Direction::Left {
        //     height_fraction = 0.75;
        // } else {
        //     width_fraction = 0.75;
        // }

        let mut rect: Rectangle = Rectangle::from_pos(self.pos);
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
    use super::*;

    fn new_grid_from_ascii(ascii: &str) -> Grid {
        let mut pos = Position::new(0, 0);
        let mut grid = Grid::new();
        for chr in ascii.chars() {
            match chr {
                '>' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                }
                '*' => {
                    grid.add_tile_connection(&pos, Direction::Right);
                    grid.add_tile_connection(&pos, Direction::Up);
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
        assert!(grid
            .reserve_position(&Reservation::new(start_pos, 0, 1))
            .is_none());

        vehicle.update(&mut grid);

        assert!(vehicle.path_status == PathStatus::Okay);
        println!("Reserved: {:?}", vehicle.reserved);

        assert!(grid
            .reserve_position(&Reservation::new(start_pos, 0, 1))
            .is_some());
    }
}
