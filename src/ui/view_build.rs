use macroquad::{
    color::{Color, RED},
    input::{is_mouse_button_down, MouseButton},
    math::Rect,
    window::{screen_height, screen_width},
};

use crate::{
    context::Context,
    map::{
        build::{action_build_road, action_one_way_road, action_two_way_road, BuildAction, BuildActionBuilding, BuildActionClearArea, BuildError, BuildResult, RoadBuildOption, TWO_WAY_ROAD_LANES}, building::Building, Map, Position
    },
    tileset::{Sprite, Tileset},
};

use super::{
    build_history::BuildHistory,
    toolbar::{Toolbar, ToolbarItem, ToolbarType, TOOLBAR_SPACE},
};

const SELECTED_HIGHLIGHT: Color = Color::new(1., 1.0, 1., 0.3);
const SELECTED_BUILD: Color = Color::new(0., 1.0, 0., 0.3);
const SELECTED_DELETE: Color = Color::new(1.0, 0., 0., 0.3);

#[derive(Clone, Copy, PartialEq)]
enum BuildMode {
    TwoWayRoad,
    OneWayRoad,
    Bridge,
    Station,
    Clear,
}

#[derive(Clone, Copy, PartialEq)]
enum BuildActions {
    Undo,
    Redo,
    Copy,
    Paste,
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
    edit_action_bar: Toolbar<BuildActions>,
    build_err: Option<BuildErrorMsg>,
    build_history: BuildHistory,
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
                    ToolbarItem::new(
                        BuildMode::Bridge,
                        "Build a bridge",
                        '3',
                        Sprite::new(8, 2),
                    ),
                    ToolbarItem::new(BuildMode::Station, "Station", '4', Sprite::new(8, 4)),
                    ToolbarItem::new(BuildMode::Clear, "Delete", '5', Sprite::new(8, 3)),
                ],
            ),
            edit_action_bar: Toolbar::new(
                ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(BuildActions::Undo, "Undo", 'u', Sprite::new(11, 0)),
                    ToolbarItem::new(BuildActions::Redo, "Redo", 'y', Sprite::new(11, 1)),
                    ToolbarItem::new(BuildActions::Copy, "Copy", 'c', Sprite::new(11, 2)),
                    ToolbarItem::new(BuildActions::Paste, "Paste", 'v', Sprite::new(11, 3)),
                ],
            ),
            build_err: None,
            build_history: BuildHistory::new(),
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

    pub fn do_action(&mut self, map: &mut Map, action: Box<dyn BuildAction>, pos: Position) {
        if let Err(err) = self.build_history.do_action(map, action) {
            self.build_err = Some(BuildErrorMsg { pos, err, time: 0 })
        }
    }

    pub fn is_mouse_over(&self, mouse_pos: (f32, f32)) -> bool {
        self.build_toolbar.is_mouse_over(mouse_pos)
    }

    pub fn mouse_clear(&mut self) {
        self.last_mouse_pos = None;
    }

    fn mouse_button_down_build(
        &mut self,
        mouse_pos: Position,
        map: &mut Map,
    ) -> Option<Box<dyn BuildAction>> {
        println!("Mouse pressed: pos: {mouse_pos:?}");
        match self.build_toolbar.get_selected()? {
            BuildMode::Clear => {
                if map.grid.is_area_clear(&mouse_pos.round_to(2), (2, 2)).is_err() {
                    Some(Box::new(BuildActionClearArea::new(mouse_pos.round_to(2), (2, 2))))
                } else {
                    None
                }
            }
            BuildMode::Station => {
                Some(Box::new(BuildActionBuilding::new(map, Building::new_station(mouse_pos, 1))))
            }
            BuildMode::Bridge => {
                if let Some(pos) = self.bridge_start_pos {
                    self.bridge_start_pos = None;
                    Some(Box::new(action_build_road(pos, mouse_pos, RoadBuildOption{
                        height: crate::map::build::BuildRoadHeight::Bridge,
                        lanes: TWO_WAY_ROAD_LANES,
                    })))
                } else {
                    self.bridge_start_pos = Some(mouse_pos);
                    None
                }
            }
            _ => None,
        }
    }

    pub fn mouse_button_down_event(&mut self, mouse_pos: Position, map: &mut Map) {
        if let Some(action) = self.mouse_button_down_build(mouse_pos, map) {
            self.do_action(map, action, mouse_pos);
        }
    }

    fn mouse_motion_build(&mut self, pos: Position, map: &mut Map) -> Option<Box<dyn BuildAction>> {
        if is_mouse_button_down(MouseButton::Left) && self.build_toolbar.get_selected().is_some() {
            println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);
            match self.build_toolbar.get_selected().unwrap() {
                BuildMode::TwoWayRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        return Some(Box::new(action_two_way_road(last_mouse_pos, pos)));
                    }
                }
                BuildMode::OneWayRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        return Some(Box::new(action_one_way_road(last_mouse_pos, pos)));
                    }
                }
                BuildMode::Clear => {
                    if map.grid.is_area_clear(&pos.round_to(2), (2, 2)).is_err() {
                        return Some(Box::new(BuildActionClearArea::new(pos.round_to(2), (2, 2))));
                    }
                }
                BuildMode::Bridge => {
                }
                BuildMode::Station => {}
            }
        }

        None
    }

    pub fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        let pos = pos.round_to(2);
        if Some(pos) == self.last_mouse_pos {
            return;
        }
        if let Some(action) = self.mouse_motion_build(pos, map) {
            self.do_action(map, action, pos);
        }
        self.last_mouse_pos = Some(pos);
    }

    pub fn key_down(&mut self, ch: char) {
        self.build_toolbar.key_down(ch);
        self.edit_action_bar.key_down(ch);
    }

    fn draw_selected(&self, last_mouse_pos: Position, _map: &Map, tileset: &Tileset) {
        let mut rect = Rect::from(last_mouse_pos);
        rect.w *= 2.;
        rect.h *= 2.;
        match self.build_toolbar.get_selected() {
            Some(BuildMode::Clear) => {
                tileset.draw_rect(&rect, SELECTED_DELETE);
            }
            Some(_) => {
                tileset.draw_rect(&rect, SELECTED_BUILD);
            }
            None => {
                tileset.draw_rect(&rect, SELECTED_HIGHLIGHT);
            }
        }
    }

    pub fn draw(&mut self, map: &Map, ctx: &Context) {
        if let Some(last_mouse_pos) = self.last_mouse_pos {
            self.draw_selected(last_mouse_pos, map, &ctx.tileset);
        }

        self.draw_build_err(&ctx.tileset);

        self.build_toolbar
            .draw(ctx, screen_width() / 2.0, screen_height() - TOOLBAR_SPACE);

        self.edit_action_bar.draw(
            ctx,
            screen_width() - (TOOLBAR_SPACE * 4.),
            screen_height() - TOOLBAR_SPACE,
        );
    }

    fn do_edit_action(&mut self, map: &mut Map, action: BuildActions) -> BuildResult {
        match action {
            BuildActions::Undo => self.build_history.undo_action(map),
            BuildActions::Redo => self.build_history.redo_action(map),
            BuildActions::Copy => todo!(),
            BuildActions::Paste => todo!(),
        }
    }

    pub fn update(&mut self, map: &mut Map) {
        self.update_build_err();

        if let Some(action) = self.edit_action_bar.get_selected().cloned() {
            if let Err(err) = self.do_edit_action(map, action) {
                self.build_err = Some(BuildErrorMsg {
                    pos: self.last_mouse_pos.unwrap_or((0, 0).into()),
                    err,
                    time: BUILD_ERROR_TIME,
                })
            }
            self.edit_action_bar.clear_selected();
        }
    }
}
