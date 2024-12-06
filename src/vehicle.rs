

use macroquad::color::Color;

use crate::grid;
use crate::grid::Direction;
use crate::grid::GridPosition;
use crate::grid::Rectangle;
use crate::path::{PathGrid, GridPath};
use crate::station;

// use std::collections::VecDeque;
// use ggez::graphics;

/// This is mostly just a semantic abstraction over a `GridPosition` to represent
/// a segment of the snake. It could be useful to, say, have each segment contain its
/// own color or something similar. This is an exercise left up to the reader ;)
#[derive(Clone, Copy, Debug)]
struct Segment {
    pos: GridPosition,
}

impl Segment {
    pub fn new(pos: GridPosition) -> Self {
        Segment { pos }
    }
}
/// Now we make a struct that contains all the information needed to describe the
/// state of the Snake itself.
pub struct Vehicle {
    head: Segment,
    dir: Direction,
    // body: VecDeque<Segment>,
    path: GridPath,
    station_id: usize,
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
            station_id: 0,
        }
    }
   

    fn update_path(&mut self, station: &station::Station, path_grid: &PathGrid) {

        self.path = path_grid.find_path(self.head.pos, station.pos);
        if self.path.is_none() {
            // couldn't find path
            println!("Couldn't find path!");
        }

    }

    /// The main update function for our snake which gets called every time
    /// we want to update the game state.
    pub fn update(&mut self, stations: &Vec<station::Station>, path_grid: &mut PathGrid) -> u32 {

        let destination = &stations[self.station_id];

        // always update path for now
        self.update_path(&destination, path_grid);


        if let Some(path) = &mut self.path {

            if path.0.is_empty() || path.0.len() < 2 {
                return 0;
            }

            let mut next_pos = path.0[0];

            if next_pos == self.head.pos {
                // path.0.remove(0);
                // println!("REMOVE");
                next_pos = path.0[1];
            }

            if path_grid.is_occupied(next_pos) {
                return 0;
            }

            // update position
            path_grid.add_occupied(next_pos);
            let prev_pos = self.head.pos;
            self.head.pos = next_pos;
            // for segment in self.body.iter_mut() {
            //     let temp = segment.pos;
            //     segment.pos = prev_pos;
            //     prev_pos = temp;
            // }
            self.dir = Direction::from_position(prev_pos, next_pos);
            path_grid.remove_occupied(prev_pos);

            // check destination
            if self.head.pos == destination.pos {
                // we made it! head to next station
                self.station_id += 1;
                if self.station_id >= stations.len() {
                    self.station_id = 0;
                }
                return 1;
            }
        }

        0
    }
    /// Here we have the Snake draw itself. This is very similar to how we saw the Food
    /// draw itself earlier.
    ///
    /// Again, note that this approach to drawing is fine for the limited scope of this
    /// example, but larger scale games will likely need a more optimized render path
    /// using `InstanceArray` or something similar that batches draw calls.
    pub fn draw(&self) {
        // We first iterate through the body segments and draw them.
        // for seg in &self.body {
        //     // Again we set the color (in this case an orangey color)
        //     // and then draw the Rect that we convert that Segment's position into
        //     canvas.draw(
        //         &graphics::Quad,
        //         graphics::DrawParam::new()
        //             .dest_rect(seg.pos.into())
        //             .color([0.6, 0.6, 0.0, 1.0]),
        //     );
        // }

        let mut width_fraction : f32= 0.9;
        let mut height_fraction: f32 = 0.9;
        if self.dir == Direction::Right || self.dir == Direction::Left {
            height_fraction = 0.75;
        } else {
            width_fraction = 0.75;
        }

        let rect: Rectangle = Rectangle::from_pos(self.head.pos, width_fraction, height_fraction);
        rect.draw(Color::from_vec([1.0, 0.1, 0.0, 1.0].into()));

        // draw the path
        // if let Some(path) = &self.path {
        //     for seg in &path.0 {
        //         // and then draw the Rect that we convert that Segment's position into
        //         canvas.draw(
        //             &graphics::Quad,
        //             graphics::DrawParam::new()
        //                 .dest_rect(seg.clone().into())
        //                 .color([0.0, 0.3, 0.0, 0.5]),
        //         ); 
        //     }
        // }
    }
}