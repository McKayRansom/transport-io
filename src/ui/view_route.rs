use macroquad::window::{screen_height, screen_width};

use crate::{context::Context, map::{Map, Position}, tileset::Sprite};

use super::toolbar::{Toolbar, ToolbarItem, TOOLBAR_SPACE};

pub struct ViewRoute {
    mouse_down_pos: Option<Position>,
    mouse_pos: Position,
    route_toolbar: Toolbar<usize>,
}

impl ViewRoute {
    pub fn new() -> Self {
        Self {
            mouse_down_pos: None,
            mouse_pos: (0, 0).into(),
            route_toolbar: Toolbar::new(
                super::toolbar::ToolbarType::Horizontal,
                (1..5)
                    .map(|i| {
                        ToolbarItem::new(
                            i as usize,
                            "",
                            char::from_digit(i - 1, 10).unwrap(),
                            Sprite::new(8, 3),
                        )
                    })
                    .collect(),
            ),
        }
    }

    pub fn draw(&mut self, map: &Map, ctx: &mut Context) {
        self.route_toolbar.draw(ctx, screen_width() / 2.0, screen_height() - TOOLBAR_SPACE);
    }
}
