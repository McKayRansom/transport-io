

use macroquad::color::Color;

use crate::grid::Direction;
use crate::grid::GridPosition;
use crate::grid::Rectangle;
use crate::grid::GRID_CELL_SIZE;
use crate::path::{PathGrid, GridPath};
use crate::station;

// use std::collections::VecDeque;
// use ggez::graphics;

const SPEED: i16 = 8;

/// This is mostly just a semantic abstraction over a `GridPosition` to represent
/// a segment of the snake. It could be useful to, say, have each segment contain its
/// own color or something similar. This is an exercise left up to the reader ;)
#[derive(Clone, Copy, Debug)]
struct Segment {
    pos: GridPosition,
    lag_pos: i16,
}

impl Segment {
    pub fn new(pos: GridPosition) -> Self {
        Segment { pos: pos, lag_pos: 0 }
    }
}
/// Now we make a struct that contains all the information needed to describe the
/// state of the Snake itself.
pub struct Vehicle {
    head: Segment,
    dir: Direction,
    // body: VecDeque<Segment>,
    path: GridPath,
    path_dirty: bool,
    station_id: usize,
    reserved: Vec<GridPosition>,
}

impl Vehicle {
    pub fn new(pos: GridPosition, path_grid: &mut PathGrid) -> Self {
        // let mut body = VecDeque::new();
        // Our snake will initially have a head and one body segment,
        // and will be moving to the right.
        // let first_segment = Segment::new((pos.x - 1, pos.y).into());
        // body.push_back(first_segment);

        assert!(path_grid.is_allowed(pos) == true);
        assert!(path_grid.is_occupied(pos) == false);
            
        path_grid.add_occupied(pos);
        // path_grid.add_occupied(first_segment.pos);

        Vehicle {
            head: Segment::new(pos),
            dir: Direction::Right,
            // body,
            path: None,
            path_dirty: true,
            station_id: 0,
            reserved: vec![pos]
        }
    }
   

    fn update_path(&mut self, station: &station::Station, path_grid: &PathGrid) {

        self.path = path_grid.find_path(self.head.pos, station.pos);
        if self.path.is_none() {
            // couldn't find path
            println!("Couldn't find path!");
        }

    }

    fn reserve_path(&mut self, path_grid: &mut PathGrid) -> bool {


        if let Some(path) = &mut self.path {
            let positions = &mut path.0;

            // reserve our route
            for pos in positions {
                if *pos == self.head.pos {
                    continue;
                }

                if path_grid.is_occupied(*pos) {
                    // failure, clear reserved
                    self.reserved.clear();
                    return false;
                }
                // reserve position (if success)
                self.reserved.push(*pos);

                // find non-intersection to wait
                if path_grid.connection_count(*pos) == 1 {
                    break;
                }
            }

            for pos in &self.reserved {
                path_grid.add_occupied(*pos);
            }

            true
        } else {
            false
        }
    }

    fn update_position(&mut self, stations: &Vec<station::Station>, path_grid: &mut PathGrid) -> u32 {

        if self.head.lag_pos > 0 {
            self.head.lag_pos -= SPEED;
        } else {

            // check destination
            if self.head.pos == stations[self.station_id].pos {
                // we made it! head to next station
                self.station_id += 1;
                if self.station_id >= stations.len() {
                    self.station_id = 0;
                }
                return 1;
            }

            for pos in &self.reserved {
                path_grid.remove_occupied(*pos);
            }
            self.reserved.clear();

            let destination = &stations[self.station_id];

            self.update_path(&destination, path_grid);

            if self.path.is_none() {
                self.reserved.push(self.head.pos);
                path_grid.add_occupied(self.head.pos);
                return 0;
            }

            if !self.reserve_path(path_grid) {
                self.reserved.push(self.head.pos);
                path_grid.add_occupied(self.head.pos);
                return 0;
            }

            if let Some(path) = &mut self.path {

                let positions = &mut path.0;
                if positions.len() == 0 {
                    return 0;
                }

                let next_pos = positions[1];
                self.head.lag_pos = (GRID_CELL_SIZE.0 as i16) - SPEED;
                self.dir = Direction::from_position(self.head.pos, next_pos);
                self.head.pos = next_pos;
            }
        }
        0
    }

    /// The main update function for our snake which gets called every time
    /// we want to update the game state.
    pub fn update(&mut self, stations: &Vec<station::Station>, path_grid: &mut PathGrid) -> u32 {

        // free our reserved route


        self.update_position(stations, path_grid)
    }

    pub fn draw(&self) {


        let mut width_fraction : f32= 0.9;
        let mut height_fraction: f32 = 0.9;
        if self.dir == Direction::Right || self.dir == Direction::Left {
            height_fraction = 0.75;
        } else {
            width_fraction = 0.75;
        }

        let mut rect: Rectangle = Rectangle::from_pos(self.head.pos, width_fraction, height_fraction);
        match self.dir {
            Direction::Right => rect.x -= self.head.lag_pos as f32,
            Direction::Left => rect.x += self.head.lag_pos as f32,
            Direction::Down => rect.y -= self.head.lag_pos as f32,
            Direction::Up => rect.y += self.head.lag_pos as f32,
        }
        rect.draw(Color::from_vec([1.0, 0.1, 0.0, 1.0].into()));

        for res in &self.reserved {
            let rect: Rectangle = Rectangle::from_pos(*res, 1.0, 1.0);
            rect.draw(Color::from_vec([1.0, 0.1, 0.0, 0.3].into()));
        }
        // draw the path
        // if let Some(path) = &self.path {
        //     for seg in &path.0 {
        //     }
        // }
    }
}