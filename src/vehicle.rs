use macroquad::color::Color;
use macroquad::color::WHITE;

use crate::grid::Direction;
use crate::grid::Position;
use crate::grid::Rectangle;
use crate::grid::GRID_CELL_SIZE;
use crate::grid::Grid;
use crate::station;
use crate::tileset::Tileset;

const SPEED: i16 = 4;

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
    destination: Position,
    reserved: Vec<Position>,
    path_status: PathStatus,
}

impl Vehicle {
    pub fn new(pos: Position, destination: Position, path_grid: &mut Grid) -> Self {
        assert!(path_grid.is_allowed(&pos) == true);
        assert!(path_grid.is_occupied(&pos) == false);

        path_grid.add_occupied(&pos);

        Vehicle {
            pos: pos,
            lag_pos: 0,
            dir: Direction::Right,
            // station_id: 0,
            destination: destination,
            reserved: vec![pos], // TODO: Safe way to do this?
            path_status: PathStatus::Okay,
        }
    }

    fn reserve(&mut self, pos: Position, path_grid: &mut Grid) {
        assert!(path_grid.is_occupied(&pos) == false);
        self.reserved.push(pos);
        path_grid.add_occupied(&pos);
    }

    pub fn clear_reserved(&mut self, path_grid: &mut Grid) {
        // shouldn't double-free
        // assert!(self.reserved.len() > 0);

        for pos in &self.reserved {
            path_grid.remove_occupied(pos);
        }
        self.reserved.clear();
    }

    fn reserve_path(&mut self, path_grid: &mut Grid, positions: &Vec<Position>) -> bool {
        // reserve our route
        for pos in positions {
            if *pos == self.pos {
                continue;
            }

            if path_grid.is_occupied(pos) {
                self.clear_reserved(path_grid);
                return false;
            }

            self.reserve(*pos, path_grid);

            // find non-intersection to wait
            if path_grid.connection_count(pos) == 1 {
                break;
            }
        }

        true
    }

    fn update_position(&mut self) {
        self.lag_pos -= SPEED;
    }

    fn update_path(&mut self, stations: &Vec<station::Station>, path_grid: &mut Grid) -> u32 {
        // check destination
        if self.pos == self.destination { //stations[self.station_id].pos {
            // we made it! head to next station
            // self.station_id += 1;
            // if self.station_id >= stations.len() {
            //     self.station_id = 0;
            // }
            return 1;
        }

        self.clear_reserved(path_grid);
        // let destination = &stations[self.station_id];

        if let Some(path) = path_grid.find_path(&self.pos, &self.destination) {
            if !self.reserve_path(path_grid, &path.0) {
                self.reserve(self.pos, path_grid);

                self.path_status = PathStatus::Waiting;
                return 0;
            }

            self.path_status = PathStatus::Okay;

            let positions = &path.0;
            if positions.len() == 0 {
                return 0;
            }

            let next_pos = positions[1];
            self.lag_pos = (GRID_CELL_SIZE.0 as i16) - SPEED;
            self.dir = Direction::from_position(self.pos, next_pos);
            self.pos = next_pos;
        } else {
            self.path_status = PathStatus::NoPath;
            self.reserve(self.pos, path_grid);
        }

        0
    }

    pub fn update(&mut self, stations: &Vec<station::Station>, path_grid: &mut Grid) -> u32 {
        if self.lag_pos > 0 {
            self.update_position();
            0
        } else {
            self.update_path(stations, path_grid)
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
        tileset.draw_tile(sprite, color, &rect, self.dir.to_radians());

        // draw the path
        // if let Some(path) = &self.path {
        //     for seg in &path.0 {
        //     }
        // }
    }
}
