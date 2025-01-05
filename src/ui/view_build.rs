use macroquad::{
    color::{Color, RED},
    input::{is_mouse_button_down, MouseButton},
    math::Rect,
    window::{screen_height, screen_width},
};

use crate::{
    context::Context, grid::{BuildError, BuildResult, Position}, map::Map, tileset::{Sprite, Tileset}
};

use super::toolbar::{Toolbar, ToolbarItem, ToolbarType, TOOLBAR_SPACE};

const SELECTED_HIGHLIGHT: Color = Color::new(1., 1.0, 1., 0.3);
const SELECTED_BUILD: Color = Color::new(0., 1.0, 0., 0.3);
const SELECTED_DELETE: Color = Color::new(1.0, 0., 0., 0.3);

#[derive(Clone, Copy, PartialEq)]
enum BuildMode {
    // Vehicle,
    // Station,
    // AddRoad,
    // RemoveRoad,
    TwoWayRoad,
    OneWayRoad,
    Bridge,
    Clear,
    // Yield,
    // Debug,
}

const BUILD_ERROR_TIME: u32 = 60 * 3;

pub struct BuildErrorMsg {
    pub pos: Position,
    pub err: BuildError,
    pub time: u32,
}

pub struct ViewBuild {
    last_mouse_pos: Option<Position>,
    bridge_start_pos: Option<Position>,
    build_toolbar: Toolbar<BuildMode>,
    build_err: Option<BuildErrorMsg>,
}

impl ViewBuild {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: None,
            bridge_start_pos: None,
            build_toolbar: Toolbar::new(
                ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(
                        BuildMode::TwoWayRoad,
                        "Build a road",
                        '1',
                        Sprite::new(8, 0),
                    ),
                    ToolbarItem::new(
                        BuildMode::OneWayRoad,
                        "Build a one way road",
                        '2',
                        Sprite::new(8, 1),
                    ),
                    ToolbarItem::new(BuildMode::Bridge, "Build a bridge", '3', Sprite::new(8, 2)),
                    ToolbarItem::new(BuildMode::Clear, "Delete", '4', Sprite::new(8, 3)),
                ],
            ),
            build_err: None,
        }
    }

    pub fn update_build_err(&mut self) {
        if let Some(build_err) = &mut self.build_err {
            build_err.time += 1;
            if build_err.time > BUILD_ERROR_TIME {
                self.build_err = None;
            }
        }
    }

    pub fn draw_build_err(&self, tileset: &Tileset) {
        if let Some(build_err) = &self.build_err {
            tileset.draw_text(
                format!("{:?}", build_err.err).as_str(),
                24.,
                RED,
                &build_err.pos.into(),
            );
        }
    }

    pub fn on_build_err(&mut self, err: BuildError, pos: Position) {
        self.build_err = Some(BuildErrorMsg { pos, err, time: 0 })
    }

    pub fn is_mouse_over(&self, mouse_pos: (f32, f32)) -> bool {
        self.build_toolbar.is_mouse_over(mouse_pos)
    }

    fn mouse_button_down_build(&mut self, mouse_pos: Position, map: &mut Map) -> BuildResult {
        println!("Mouse pressed: pos: {mouse_pos:?}");
        let build_mode = self.build_toolbar.get_selected();
        if build_mode.is_none() {
            return Ok(());
        }
        match build_mode.unwrap() {
            BuildMode::Clear => {
                map.clear_area(&mouse_pos)?;
            }
            // BuildMode::Yield => {
            //     if let Some(Tile::Road(road)) = map.grid.get_tile_mut(&mouse_pos) {
            //         road.should_yield = !road.should_yield;
            //     }
            // }
            BuildMode::Bridge => {
                if let Some(start_pos) = self.bridge_start_pos {
                    self.bridge_start_pos = None;
                    map.grid.build_bridge(start_pos, mouse_pos)?;
                } else {
                    self.bridge_start_pos = Some(mouse_pos);
                }
            }
            // BuildMode::Roundabout => {
            //     let roundabout = map.add_intersection();
            //     for tile: Tile in map.path_grid.get_tiles(pos, 2) {
            //         if let Tile::Road(road) = tile {
            //             road.intersection = roundabout;
            //         }
            //     }
            // }
            _ => {}
        }

        Ok(())
    }

    pub fn mouse_button_down_event(&mut self, mouse_pos: Position, map: &mut Map) {
        if let Err(err) = self.mouse_button_down_build(mouse_pos, map) {
            self.on_build_err(err, mouse_pos);
        }
    }

    fn mouse_motion_build(&mut self, pos: Position, map: &mut Map) -> BuildResult {
        if is_mouse_button_down(MouseButton::Left) && self.build_toolbar.get_selected().is_some() {
            match self.build_toolbar.get_selected().unwrap() {
                // BuildMode::AddRoad => {
                //     if let Some(last_mouse_pos) = self.last_mouse_pos {
                //         let dir = last_mouse_pos.direction_to(pos);
                //         map.grid.build_road(&last_mouse_pos, dir)?
                //     }
                // }
                // BuildMode::RemoveRoad => {
                //     if let Some(last_mouse_pos) = self.last_mouse_pos {
                //         let dir = last_mouse_pos.direction_to(pos);
                //         map.grid.remove_road(&last_mouse_pos, dir)?
                //     }
                // }
                BuildMode::TwoWayRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        let dir = last_mouse_pos.direction_to(pos);
                        map.grid.build_two_way_road(last_mouse_pos, dir)?;
                    }
                }
                BuildMode::OneWayRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        let dir = last_mouse_pos.direction_to(pos);
                        map.grid.build_one_way_road(last_mouse_pos, dir)?;
                    }
                }
                BuildMode::Clear => {
                    map.clear_area(&pos)?;
                }
                _ => {}
            }
        }
        println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);

        Ok(())
    }

    pub fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        let pos = pos.round_to(2);
        if Some(pos) == self.last_mouse_pos {
            return;
        }
        if let Err(err) = self.mouse_motion_build(pos, map) {
            self.on_build_err(err, pos);
        }
        self.last_mouse_pos = Some(pos);
    }

    pub fn key_down(&mut self, ch: char) {
        self.build_toolbar.key_down(ch);
    }

    fn draw_selected(&self, last_mouse_pos: Position, _map: &Map, tileset: &Tileset) {
        // draw selected
        let mut rect = Rect::from(last_mouse_pos);
        rect.w *= 2.;
        rect.h *= 2.;
        match self.build_toolbar.get_selected() {
            Some(BuildMode::Bridge) => {
                if let Some(start_pos) = self.bridge_start_pos {
                    for pos in start_pos.iter_line_to(last_mouse_pos).0 {
                        tileset.draw_rect(&Rect::from(pos), SELECTED_BUILD);
                    }
                } else {
                    tileset.draw_rect(&rect, SELECTED_BUILD);
                }
            }
            // Some(BuildMode::Debug) => {
            //     if let Some((path, _cost)) = map.grid.find_road(&last_mouse_pos) {
            //         for pos in path {
            //             tileset.draw_rect(&Rect::from(pos), SELECTED_DELETE);
            //         }
            //     }
            // }
            Some(BuildMode::Clear) => {
                tileset.draw_rect(&rect, SELECTED_DELETE);
            }
            Some(BuildMode::TwoWayRoad) => {
                tileset.draw_rect(&rect, SELECTED_BUILD);
            }
            _ => {
                tileset.draw_rect(&rect, SELECTED_HIGHLIGHT);
            }
        }
    }

    pub fn draw(&mut self, map: &Map, ctx: &Context) {
        if let Some(last_mouse_pos) = self.last_mouse_pos {
            self.draw_selected(last_mouse_pos, map, &ctx.tileset);
        }

        self.draw_build_err(&ctx.tileset);

        self.build_toolbar.draw(
            ctx,
            screen_width() / 2.0,
            screen_height() - TOOLBAR_SPACE,
        );
    }

    pub fn update(&mut self) {
        self.update_build_err();
    }
}
