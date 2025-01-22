use macroquad::{
    color::{Color, RED},
    window::{screen_height, screen_width},
};

use crate::{
    context::Context,
    map::{build::*, Direction, Map, Position, Unlocked},
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
    mouse_down_pos: Option<Position>,
    mouse_pos: Position,
    build_toolbar: Toolbar<BuildMode>,
    edit_action_bar: Toolbar<BuildActions>,
    build_err: Option<BuildErrorMsg>,
    build_history: BuildHistory,
}

impl ViewBuild {
    pub fn new(unlocked: Unlocked) -> Self {
        let mut build_toolbar: Vec<ToolbarItem<BuildMode>> = Vec::new();
        build_toolbar.push(ToolbarItem::new(
            BuildMode::TwoWayRoad,
            "Build a road",
            '1',
            Sprite::new(8, 0),
        ));
        if unlocked == Unlocked::OneWayRoads || unlocked == Unlocked::All {
            build_toolbar.push(ToolbarItem::new(
                BuildMode::OneWayRoad,
                "Build a one way road",
                '2',
                Sprite::new(8, 1),
            ))
        }

        if unlocked != Unlocked::Roads {
            build_toolbar.push(ToolbarItem::new(
                BuildMode::Bridge,
                "Build a bridge",
                '3',
                Sprite::new(8, 2),
            ))
        }
        // ToolbarItem::new(BuildMode::Station, "Station", '4', Sprite::new(8, 4)),

        build_toolbar.push(ToolbarItem::new(
            BuildMode::Clear,
            "Delete",
            '5',
            Sprite::new(8, 3),
        ));

        Self {
            mouse_pos: Position::new(0, 0),
            mouse_down_pos: None,
            build_toolbar: Toolbar::new(ToolbarType::Horizontal, build_toolbar),
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
        self.build_toolbar.is_mouse_over(mouse_pos) || self.edit_action_bar.is_mouse_over(mouse_pos)
    }

    pub fn mouse_clear(&mut self) {
        self.mouse_down_pos = None;
    }

    fn mouse_button_up_build(
        &mut self,
        pos: Position,
        map: &mut Map,
    ) -> Option<Box<dyn BuildAction>> {
        let pos = pos.round_to(2);
        match self.build_toolbar.get_selected()? {
            BuildMode::TwoWayRoad => Some(Box::new(action_two_way_road(self.mouse_down_pos?, pos))),
            BuildMode::OneWayRoad => Some(Box::new(action_one_way_road(self.mouse_down_pos?, pos))),
            BuildMode::Clear => {
                let area: Direction = pos - self.mouse_down_pos?.round_to(2);
                if map.grid.is_area_clear(&pos, area).is_err() {
                    Some(Box::new(BuildActionClearArea::new(
                        self.mouse_down_pos?,
                        area,
                    )))
                } else {
                    None
                }
            }
            BuildMode::Bridge => Some(Box::new(action_build_road(
                self.mouse_down_pos?,
                pos,
                RoadBuildOption {
                    height: crate::map::build::BuildRoadHeight::Bridge,
                    lanes: TWO_WAY_ROAD_LANES,
                },
            ))),
            BuildMode::Station => Some(Box::new(action_build_road(
                self.mouse_down_pos?,
                pos,
                RoadBuildOption {
                    height: crate::map::build::BuildRoadHeight::Bridge,
                    lanes: TWO_WAY_ROAD_LANES,
                },
            ))),
        }
    }

    pub fn mouse_button_down_event(&mut self, mouse_pos: Position, _map: &mut Map) {
        self.mouse_down_pos = Some(mouse_pos);
    }

    pub fn mouse_button_up_event(&mut self, mouse_pos: Position, map: &mut Map) {
        // println!("Mouse Up, x: {}, y: {}", pos.x, pos.y);

        if let Some(action) = self.mouse_button_up_build(mouse_pos, map) {
            self.do_action(map, action, mouse_pos);
        }

        self.mouse_down_pos = None;
    }

    pub fn mouse_motion_event(&mut self, pos: Position, _map: &mut Map) {
        self.mouse_pos = pos;
    }

    pub fn key_down(&mut self, ch: char) {
        self.build_toolbar.key_down(ch);
        self.edit_action_bar.key_down(ch);
    }

    fn draw_selected(&self, mouse_down_pos: Position, _map: &Map, tileset: &Tileset) {
        let start_pos = mouse_down_pos.round_to(2);
        let end_pos = self.mouse_pos.round_to(2);
        let dir = start_pos.direction_to(end_pos);
        match self.build_toolbar.get_selected() {
            Some(BuildMode::Clear) => {
                for pos in start_pos.iter_area(end_pos - start_pos) {
                    tileset.draw_rect(&pos.into(), SELECTED_DELETE);
                }
            }
            Some(_) => {
                let (pos_iter, _) = start_pos
                    .corner_pos(dir)
                    .iter_line_to(end_pos.corner_pos(dir.inverse()));
                for pos in pos_iter {
                    tileset.draw_rect(&pos.into(), SELECTED_BUILD);
                    tileset.draw_rect(&(pos + dir.rotate_right()).into(), SELECTED_BUILD);
                }
            }
            None => {
                tileset.draw_rect(&start_pos.into(), SELECTED_HIGHLIGHT);
            }
        }
    }

    pub fn draw(&mut self, map: &Map, ctx: &Context) {
        if let Some(mouse_down_pos) = self.mouse_down_pos {
            self.draw_selected(mouse_down_pos, map, &ctx.tileset);
        } else {
            ctx.tileset
                .draw_rect(&self.mouse_pos.into(), SELECTED_HIGHLIGHT);
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
                    pos: self.mouse_down_pos.unwrap_or((0, 0).into()),
                    err,
                    time: BUILD_ERROR_TIME,
                })
            }
            self.edit_action_bar.clear_selected();
        }
    }
}
