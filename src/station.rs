
use crate::grid::{GridPosition, Rectangle};

/// This is again an abstraction over a `GridPosition` that represents
/// a piece of food the snake can eat. It can draw itself.
pub struct Station {
    pub pos: GridPosition,
}

impl Station {
    pub fn new(pos: GridPosition) -> Self {
        Station { pos }
    }

    /// Here is the first time we see what drawing looks like with ggez.
    /// We have a function that takes in a `&mut ggez::graphics::Canvas` which we use
    /// to do drawing.
    ///
    /// Note: this method of drawing does not scale. If you need to render
    /// a large number of shapes, use an `InstanceArray`. This approach is fine for
    /// this example since there are a fairly limited number of calls.
    pub fn draw(&self) {
        // First we set the color to draw with, in this case all food will be
        // colored blue.
        let color = [0.0, 0.0, 1.0, 1.0];
        // Then we draw a rectangle with the Fill draw mode, and we convert the
        // Food's position into a `ggez::Rect` using `.into()` which we can do
        // since we implemented `From<GridPosition>` for `Rect` earlier.
        let rect:Rectangle = self.pos.into();
        rect.draw(macroquad::color::Color::from_vec(color.into()));
    }
}
