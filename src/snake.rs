

use crate::grid::Direction;
use crate::grid::GridPosition;
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
    /// Finally we store the direction that the snake was traveling the last
    /// time that `update` was called, which we will use to determine valid
    /// directions that it could move the next time update is called.
    last_update_dir: Direction,
    /// Store the direction that will be used in the `update` after the next `update`
    /// This is needed so a user can press two directions (eg. left then up)
    /// before one `update` has happened. It sort of queues up key press input
    next_dir: Option<Direction>,
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
            last_update_dir: Direction::Right,
            body,
            ate: None,
            next_dir: None,
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

    /// The main update function for our snake which gets called every time
    /// we want to update the game state.
    pub fn update(&mut self, food: &food::Food) {

        // set dir
        if food.pos.x > self.head.pos.x {
            self.dir = Direction::Right;
        } else if food.pos.x < self.head.pos.x {
            self.dir = Direction::Left; 
        } else if food.pos.y < self.head.pos.y {
            self.dir = Direction::Up;
        } else if food.pos.y > self.head.pos.y {
            self.dir = Direction::Down;
        }


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
        // And set our last_update_dir to the direction we just moved.
        self.last_update_dir = self.dir;
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
                    .color([0.3, 0.3, 0.0, 1.0]),
            );
        }
        // And then we do the same for the head, instead making it fully red to distinguish it.
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest_rect(self.head.pos.into())
                .color([1.0, 0.5, 0.0, 1.0]),
        );
    }

    pub fn keydir(&mut self, dir: Direction)
    {
        // If it succeeds, we check if a new direction has already been set
        // and make sure the new direction is different then `snake.dir`
        if self.dir != self.last_update_dir && dir.inverse() != self.dir {
            self.next_dir = Some(dir);
        } else if dir.inverse() != self.last_update_dir {
            // If no new direction has been set and the direction is not the inverse
            // of the `last_update_dir`, then set the snake's new direction to be the
            // direction the user pressed.
            self.dir = dir;
        } 
    }
}