

use crate::grid::Direction;
use crate::grid::GridPosition;
use crate::path::{find_path, GridPath};
use crate::food;

use std::collections::VecDeque;
use ggez::graphics;

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

/// Here we define an enum of the possible things that the snake could have "eaten"
/// during an update of the game. It could have either eaten a piece of `Food`, or
/// it could have eaten `Itself` if the head ran into its body.
#[derive(Clone, Copy, Debug)]
pub enum Ate {
    Itself,
    Food,
}

/// Now we make a struct that contains all the information needed to describe the
/// state of the Snake itself.
pub struct Snake {
    /// First we have the head of the snake, which is a single `Segment`.
    head: Segment,
    /// Then we have the current direction the snake is moving. This is
    /// the direction it will move when `update` is called on it.
    dir: Direction,
    /// Next we have the body, which we choose to represent as a `VecDeque`
    /// of `Segment`s.
    body: VecDeque<Segment>,
    /// Now we have a property that represents the result of the last update
    /// that was performed. The snake could have eaten nothing (None), Food (Some(Ate::Food)),
    /// or Itself (Some(Ate::Itself))
    pub ate: Option<Ate>,

    path: GridPath,
}

impl Snake {
    pub fn new(pos: GridPosition) -> Self {
        let mut body = VecDeque::new();
        // Our snake will initially have a head and one body segment,
        // and will be moving to the right.
        body.push_back(Segment::new((pos.x - 1, pos.y).into()));
        Snake {
            head: Segment::new(pos),
            dir: Direction::Right,
            body,
            ate: None,
            path: None,
        }
    }

    /// A helper function that determines whether
    /// the snake eats a given piece of Food based
    /// on its current position
    fn eats(&self, food: &food::Food) -> bool {
        self.head.pos == food.pos
    }

    /// A helper function that determines whether
    /// the snake eats itself based on its current position
    fn eats_self(&self) -> bool {
        for seg in &self.body {
            if self.head.pos == seg.pos {
                return true;
            }
        }
        false
    }

    fn update_path(&mut self, food: &food::Food) {
        // find path
        if self.path.is_none() {
            let segments: Vec<GridPosition> = self.body
                .iter()
                .map(|segment| segment.pos)
                .collect();

            self.path = find_path(self.head.pos, food.pos, &segments);
            if self.path.is_none() {
                // couldn't find path
                println!("Couldn't find path!");
            }
        }

        if let Some(path) = &mut self.path {

            if path.0.is_empty() {
                return;
            }

            let mut next_pos = path.0[0];

            if next_pos == self.head.pos {
                path.0.remove(0);
            }

            if path.0.is_empty() {
                return;
            }

            next_pos = path.0[0];

            // set dir
            if next_pos.x > self.head.pos.x {
                self.dir = Direction::Right;
            } else if next_pos.x < self.head.pos.x {
                self.dir = Direction::Left; 
            } else if next_pos.y < self.head.pos.y {
                self.dir = Direction::Up;
            } else if next_pos.y > self.head.pos.y {
                self.dir = Direction::Down;
            }
        }
    }

    /// The main update function for our snake which gets called every time
    /// we want to update the game state.
    pub fn update(&mut self, food: &food::Food) {


        self.update_path(food);


        // First we get a new head position by using our `new_from_move` helper
        // function from earlier. We move our head in the direction we are currently
        // heading.
        let new_head_pos = GridPosition::new_from_move(self.head.pos, self.dir);
        // Next we create a new segment will be our new head segment using the
        // new position we just made.
        let new_head = Segment::new(new_head_pos);
        // Then we push our current head Segment onto the front of our body
        self.body.push_front(self.head);
        // And finally make our actual head the new Segment we created. This has
        // effectively moved the snake in the current direction.
        self.head = new_head;
        // Next we check whether the snake eats itself or some food, and if so,
        // we set our `ate` member to reflect that state.
        if self.eats_self() {
            self.ate = Some(Ate::Itself);
        } else if self.eats(food) {
            self.ate = Some(Ate::Food);
            self.path = None;
        } else {
            self.ate = None;
        }
        // If we didn't eat anything this turn, we remove the last segment from our body,
        // which gives the illusion that the snake is moving. In reality, all the segments stay
        // stationary, we just add a segment to the front and remove one from the back. If we eat
        // a piece of food, then we leave the last segment so that we extend our body by one.
        if self.ate.is_none() {
            self.body.pop_back();
        }
    }

    /// Here we have the Snake draw itself. This is very similar to how we saw the Food
    /// draw itself earlier.
    ///
    /// Again, note that this approach to drawing is fine for the limited scope of this
    /// example, but larger scale games will likely need a more optimized render path
    /// using `InstanceArray` or something similar that batches draw calls.
    pub fn draw(&self, canvas: &mut graphics::Canvas) {
        // We first iterate through the body segments and draw them.
        for seg in &self.body {
            // Again we set the color (in this case an orangey color)
            // and then draw the Rect that we convert that Segment's position into
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest_rect(seg.pos.into())
                    .color([0.3, 0.6, 0.0, 1.0]),
            );
        }
        // And then we do the same for the head, instead making it fully red to distinguish it.
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest_rect(self.head.pos.into())
                .color([1.0, 0.5, 0.0, 1.0]),
        );

        // draw the path
        if let Some(path) = &self.path {
            for seg in &path.0 {
                // and then draw the Rect that we convert that Segment's position into
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest_rect(seg.clone().into())
                        .color([0.1, 0.9, 0.0, 0.5]),
                ); 
            }

        }
    }

    pub fn keydir(&mut self, dir: Direction)
    {
        // If it succeeds, we check if a new direction has already been set
        // and make sure the new direction is different then `snake.dir`

        // If no new direction has been set and the direction is not the inverse
        // of the `last_update_dir`, then set the snake's new direction to be the
        // direction the user pressed.
        self.dir = dir;
    }
}