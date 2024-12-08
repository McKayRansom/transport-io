use macroquad::color::WHITE;

use crate::{grid::{Position, Rectangle}, tileset::Tileset};

pub struct Station {
    pub pos: Position,
}

// const STATION_COLOR: Color = Color::new(0.0, 0.0, 1.0, 1.0);
const STATION_SPRITE: u32 = (16 * 4) + 0;

impl Station {
    pub fn _new(pos: Position) -> Self {
        Station { pos }
    }

    /// Note: this method of drawing does not scale. If you need to render
    /// a large number of shapes, use an `InstanceArray`. This approach is fine for
    /// this example since there are a fairly limited number of calls.
    pub fn draw(&self, tileset: &Tileset) {

        let rect: Rectangle = Rectangle::from_pos(self.pos);
        // rect.draw(STATION_COLOR);
        tileset.draw_tile(STATION_SPRITE, WHITE, &rect, 0.);

    }
}
