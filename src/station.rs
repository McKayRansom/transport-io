use macroquad::color::Color;

use crate::grid::{Position, Rectangle};

pub struct Station {
    pub pos: Position,
}

const STATION_COLOR: Color = Color::new(0.0, 0.0, 1.0, 1.0);

impl Station {
    pub fn new(pos: Position) -> Self {
        Station { pos }
    }

    /// Note: this method of drawing does not scale. If you need to render
    /// a large number of shapes, use an `InstanceArray`. This approach is fine for
    /// this example since there are a fairly limited number of calls.
    pub fn draw(&self) {

        let rect: Rectangle = Rectangle::from_pos(self.pos, 0.5, 0.5);
        rect.draw(STATION_COLOR);
    }
}
