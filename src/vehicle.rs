use macroquad::color::Color;

use crate::grid::Direction;
use crate::grid::Position;
use crate::grid::Rectangle;
use crate::grid::GRID_CELL_SIZE;
use crate::grid::Grid;
use crate::station;

const SPEED: i16 = 8;

const OKAY_COLOR: Color = Color::new(0.0, 0.9, 0.1, 1.0);
const NO_PATH_COLOR: Color = Color::new(1.0, 0.1, 0.0, 1.0);

pub struct Vehicle {
    pos: Position,
    lag_pos: i16,
    dir: Direction,
    station_id: usize,
    reserved: Vec<Position>,
    no_path: bool,
}

impl Vehicle {
    pub fn new(pos: Position, path_grid: &mut Grid) -> Self {
        assert!(path_grid.is_allowed(&pos) == true);
        assert!(path_grid.is_occupied(&pos) == false);

        path_grid.add_occupied(&pos);

        Vehicle {
            pos: pos,
            lag_pos: 0,
            dir: Direction::Right,
            station_id: 0,
            reserved: vec![pos], // TODO: Safe way to do this?
            no_path: false,
        }
    }

    fn reserve(&mut self, pos: Position, path_grid: &mut Grid) {
        assert!(path_grid.is_occupied(&pos) == false);
        self.reserved.push(pos);
        path_grid.add_occupied(&pos);
    }

    fn clear_reserved(&mut self, path_grid: &mut Grid) {
        // shouldn't double-free
        assert!(self.reserved.len() > 0);

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
        if self.pos == stations[self.station_id].pos {
            // we made it! head to next station
            self.station_id += 1;
            if self.station_id >= stations.len() {
                self.station_id = 0;
            }
            return 1;
        }

        self.clear_reserved(path_grid);
        let destination = &stations[self.station_id];

        if let Some(path) = path_grid.find_path(&self.pos, &destination.pos) {
            if !self.reserve_path(path_grid, &path.0) {
                self.reserve(self.pos, path_grid);
                return 0;
            }

            let positions = &path.0;
            if positions.len() == 0 {
                return 0;
            }

            let next_pos = positions[1];
            self.lag_pos = (GRID_CELL_SIZE.0 as i16) - SPEED;
            self.dir = Direction::from_position(self.pos, next_pos);
            self.pos = next_pos;
        } else {
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

    pub fn draw(&self) {

        let mut width_fraction: f32 = 0.9;
        let mut height_fraction: f32 = 0.9;
        if self.dir == Direction::Right || self.dir == Direction::Left {
            height_fraction = 0.75;
        } else {
            width_fraction = 0.75;
        }

        let mut rect: Rectangle = Rectangle::from_pos(self.pos, width_fraction, height_fraction);
        match self.dir {
            Direction::Right => rect.x -= self.lag_pos as f32,
            Direction::Left => rect.x += self.lag_pos as f32,
            Direction::Down => rect.y -= self.lag_pos as f32,
            Direction::Up => rect.y += self.lag_pos as f32,
        }

        let color = if self.no_path {
            NO_PATH_COLOR
        } else {
            OKAY_COLOR
        };
        rect.draw(color);

        // draw the path
        // if let Some(path) = &self.path {
        //     for seg in &path.0 {
        //     }
        // }
    }
}
