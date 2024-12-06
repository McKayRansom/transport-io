
use macroquad::input::KeyCode;
use macroquad::shapes::draw_rectangle;
use macroquad::color::Color;

// use ggez::{graphics, input::keyboard::KeyCode};

// Here we define the size of our game board in terms of how many grid
// cells it will take up. We choose to make a 30 x 20 game board.
pub const GRID_SIZE: (i16, i16) = (30, 20);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
pub const GRID_CELL_SIZE: (f32, f32) = (32., 32.);


#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Hash)]
pub struct GridPosition {
    pub x: i16,
    pub y: i16,
}

impl GridPosition {
    /// We make a standard helper function so that we can create a new `GridPosition`
    /// more easily.
    pub fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }

    pub fn from_screen(x: f32, y: f32) -> Self {
        GridPosition {
            x: x as i16 / GRID_CELL_SIZE.0 as i16,
            y: y as i16 / GRID_CELL_SIZE.1 as i16,
        }
    }

    pub fn valid(&self) -> bool {
        self.x > 0 && self.y > 0 && self.x < GRID_SIZE.0 && self.y < GRID_SIZE.1
    }

    /// We'll make another helper function that takes one grid position and returns a new one after
    /// making one move in the direction of `dir`.
    /// We use the [`rem_euclid()`](https://doc.rust-lang.org/std/primitive.i16.html#method.rem_euclid)
    /// API when crossing the top/left limits, as the standard remainder function (`%`) returns a
    /// negative value when the left operand is negative.
    /// Only the Up/Left cases require rem_euclid(); for consistency, it's used for all of them.
    pub fn new_from_move(pos: GridPosition, dir: Direction) -> Self {
        match dir {
            Direction::Up => GridPosition::new(pos.x, (pos.y - 1).rem_euclid(GRID_SIZE.1)),
            Direction::Down => GridPosition::new(pos.x, (pos.y + 1).rem_euclid(GRID_SIZE.1)),
            Direction::Left => GridPosition::new((pos.x - 1).rem_euclid(GRID_SIZE.0), pos.y),
            Direction::Right => GridPosition::new((pos.x + 1).rem_euclid(GRID_SIZE.0), pos.y),
        }
    }
}


pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rectangle{x, y, w, h}
    }

    pub fn from_pos(pos: GridPosition, width_fraction: f32, height_fraction: f32) -> Self {
        Rectangle::new(
            (pos.x as f32 * GRID_CELL_SIZE.0) + (GRID_CELL_SIZE.0 * (1.0 - width_fraction)/2.0),
            (pos.y as f32 * GRID_CELL_SIZE.1) + (GRID_CELL_SIZE.1 * (1.0 - height_fraction)/2.0),
            (GRID_CELL_SIZE.0 as f32) * width_fraction,
            (GRID_CELL_SIZE.1 as f32) * height_fraction,
        )
    }


    pub fn draw(&self, color: Color) {

        draw_rectangle(
            self.x,
            self.y,
            self.w,
            self.h,
            color,
        );
    }

}

/// And here we implement `From` again to allow us to easily convert between
/// `(i16, i16)` and a `GridPosition`.
impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

/// Next we create an enum that will represent all the possible
/// directions that our snake could move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up = 1,
    Down = 2,
    Left = 4,
    Right = 8,
}

impl Direction {
    /// We create a helper function that will allow us to easily get the inverse
    /// of a `Direction` which we can use later to check if the player should be
    /// able to move the snake in a certain direction.
    pub fn inverse(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn rotate(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn from_position(pos1: GridPosition, pos2: GridPosition) -> Self {
        if pos2.x > pos1.x {
            Direction::Right
        } else if pos2.y > pos1.y {
            Direction::Down
        } else if pos2.y < pos1.y {
            Direction::Down
        } else {
            Direction::Left
        }
    }

    /// We also create a helper function that will let us convert between a
    /// `ggez` `Keycode` and the `Direction` that it represents. Of course,
    /// not every keycode represents a direction, so we return `None` if this
    /// is the case.
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}
