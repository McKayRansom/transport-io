use macroquad::{
    math::Rect,
    window::{screen_height, screen_width},
};

use crate::{
    consts::SpawnerColors,
    context::Context,
    map::{route::RouteStop, tile::Tile, Map, Position},
    tileset::Sprite,
};

use super::toolbar::{Toolbar, ToolbarItem, TOOLBAR_SPACE};

pub struct ViewRoute {
    mouse_down_pos: Option<Position>,
    route_toolbar: Toolbar<usize>,
}

impl ViewRoute {
    pub fn new() -> Self {
        Self {
            mouse_down_pos: None,
            route_toolbar: Toolbar::new(
                super::toolbar::ToolbarType::Horizontal,
                (1..5)
                    .map(|i| ToolbarItem {
                        value: i as usize,
                        shortcut: char::from_digit(i - 1, 10).unwrap(),
                        sprite: Sprite::new(13, 0),
                        color: SpawnerColors::from_number(i as u64).color(),
                        ..Default::default()
                    })
                    .collect(),
            ),
        }
    }

    pub fn draw(&mut self, map: &Map, ctx: &mut Context) {
        self.route_toolbar
            .draw(ctx, screen_width() / 2.0, screen_height() - TOOLBAR_SPACE);

        if let Some(route_numer) = self.route_toolbar.get_selected() {
            if let Some(mouse_coords) = ctx.mouse_pos {
                let pos = Position::from_screen(
                    mouse_coords.into(),
                    ctx.tileset.camera,
                    ctx.tileset.zoom,
                );
                if let Some((stop, _index)) = map.routes[*route_numer].closest_stop(pos) {
                    let start: Rect = stop.pos.into();
                    let end: Rect = pos.into();
                    ctx.tileset.draw_line(
                        &start,
                        &end,
                        10.,
                        SpawnerColors::from_number(*route_numer as u64).color(),
                    );
                }
            }
        }
    }

    pub fn mouse_button_down_event(&mut self, mouse_pos: Position, _map: &mut Map) {
        self.mouse_down_pos = Some(mouse_pos);
    }

    pub fn mouse_button_up_event(&mut self, mouse_pos: Position, map: &mut Map) {
        if let Some(route_number) = self.route_toolbar.get_selected() {
            if let Some(Tile::Road(road)) = map.grid.get_tile(&mouse_pos) {
                if let Some(station) = road.station {
                    map.routes[*route_number].add_stop(RouteStop {
                        station: station,
                        pos: mouse_pos,
                    });
                }
            }
        }
        self.mouse_down_pos = None;
    }
}
