use macroquad::{
    color::{Color, RED},
    window::{screen_height, screen_width},
};

use crate::{
    context::Context,
    map::{build::*, building::Building, tile::Tile, Direction, Map, Position, Unlocked},
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
}

#[derive(Clone, Copy, PartialEq)]
enum BuildActions {
    Undo,
    Redo,
    Clear,
    Copy,
    Cut,
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
    clipboard: Option<Clipboard>,
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

        if unlocked == Unlocked::All {
            build_toolbar.push(ToolbarItem::new(
                BuildMode::Station,
                "Station",
                '4',
                Sprite::new(8, 4),
            ));
        }

        Self {
            mouse_pos: Position::new(0, 0),
            mouse_down_pos: None,
            build_toolbar: Toolbar::new(ToolbarType::Horizontal, build_toolbar),
            edit_action_bar: Toolbar::new(
                ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(BuildActions::Undo, "Undo", 'z', Sprite::new(11, 0)),
                    ToolbarItem::new(BuildActions::Redo, "Redo", 'y', Sprite::new(11, 1)),
                    ToolbarItem::new(BuildActions::Clear, "Clear", '\u{7f}', Sprite::new(11, 2)),
                    ToolbarItem::new(BuildActions::Copy, "Copy", 'c', Sprite::new(11, 3)),
                    ToolbarItem::new(BuildActions::Cut, "Cut", 'x', Sprite::new(11, 4)),
                    ToolbarItem::new(BuildActions::Paste, "Paste", 'v', Sprite::new(11, 5)),
                ],
            ),
            build_err: None,
            build_history: BuildHistory::new(),
            clipboard: None,
        }
    }

    fn update_build_err(&mut self) {
        if let Some(build_err) = &mut self.build_err {
            build_err.time += 1;
            if build_err.time > BUILD_ERROR_TIME {
                self.build_err = None;
            }
        }
    }

    fn draw_build_err(&self, tileset: &Tileset) {
        if let Some(build_err) = &self.build_err {
            tileset.draw_text(
                format!("{:?}", build_err.err).as_str(),
                24.,
                RED,
                &build_err.pos.into(),
            );
        }
    }

    fn do_action(&mut self, map: &mut Map, action: Box<dyn BuildAction>, pos: Position) {
        if let Err(err) = self.build_history.do_action(map, action) {
            self.build_err = Some(BuildErrorMsg { pos, err, time: 0 })
        }
    }

    pub fn mouse_clear(&mut self) {
        self.mouse_down_pos = None;
    }

    fn mouse_button_up_build(
        &mut self,
        pos: Position,
        map: &mut Map,
    ) -> Option<Box<dyn BuildAction>> {

        let start_pos = self.mouse_down_pos?;
        let end_pos = pos + Direction::new(1, 1, 0);
        let area: Direction = end_pos - start_pos;

        match self.edit_action_bar.get_selected() {
            Some(BuildActions::Clear) => {
                if map.grid.is_area_clear(&start_pos, area).is_err() {
                    return Some(Box::new(BuildActionClearArea::new(start_pos, area)));
                } else {
                    return None;
                }
            },
            Some(BuildActions::Copy) => {
                self.clipboard = Some(Clipboard::new(map, start_pos, area));
                self.edit_action_bar.clear_selected();
                return None;
            },
            Some(BuildActions::Cut) => {
                self.clipboard = Some(Clipboard::new(map, start_pos, area));
                self.edit_action_bar.clear_selected();
                return Some(Box::new(BuildActionClearArea::new(start_pos, area)));
            }
            Some(BuildActions::Paste) => {
                self.edit_action_bar.clear_selected();
                if let Some(clipboard) = &self.clipboard {
                    return Some(Box::new(clipboard.paste(start_pos)));
                } else {
                    return None;
                }
            }
            _ => {}
        }
        match self.build_toolbar.get_selected()? {
            BuildMode::TwoWayRoad => Some(Box::new(action_two_way_road(start_pos, pos))),
            BuildMode::OneWayRoad => Some(Box::new(action_one_way_road(start_pos, pos))),
            BuildMode::Station => Some(Box::new(BuildActionStation::new(
                map,
                pos,
                Building::new_station(pos),
            ))),
            BuildMode::Bridge => Some(Box::new(action_build_road(
                start_pos,
                pos,
                RoadBuildOption {
                    height: crate::map::build::BuildRoadHeight::Bridge,
                    one_way: false,
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

    pub fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        if let Some(tile) = map.grid.get_tile(&pos) {
            dbg!(tile);
            if let Tile::Building(id) = tile {
                if let Some(building) = map.get_building(id) {
                    dbg!(id);
                    dbg!(building);
                }
            }
            for vehicle in map.vehicles.hash_map.values() {
                if vehicle.pos.pos == pos {
                    dbg!(vehicle);
                }
            }
        }
        self.mouse_pos = pos;
    }

    fn draw_selected(&self, mouse_down_pos: Position, _map: &Map, tileset: &Tileset) {
        let start_pos = mouse_down_pos;
        let end_pos = self.mouse_pos;
        match self.edit_action_bar.get_selected() {
            Some(_) => {
                let end_pos = end_pos + Direction::new(1, 1, 0);
                for pos in start_pos.iter_area(end_pos - start_pos) {
                    tileset.draw_rect(&pos.into(), SELECTED_DELETE);
                }
                return;
            }
            _ => {}
        }
        match self.build_toolbar.get_selected() {
            Some(_) => {
                let (pos_iter, _) = start_pos.iter_line_to(end_pos);
                for pos in pos_iter {
                    tileset.draw_rect(&pos.into(), SELECTED_BUILD);
                }
            }
            None => {
                tileset.draw_rect(&start_pos.into(), SELECTED_HIGHLIGHT);
            }
        }
    }

    pub fn draw(&mut self, map: &mut Map, ctx: &mut Context) {
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
            screen_width() - (TOOLBAR_SPACE * 3.),
            screen_height() - TOOLBAR_SPACE,
        );
    }

    fn do_edit_action(&mut self, map: &mut Map, action: BuildActions) -> BuildResult {
        match action {
            BuildActions::Undo => {
                self.edit_action_bar.clear_selected();
                self.build_history.undo_action(map)
            }
            BuildActions::Redo => {
                self.edit_action_bar.clear_selected();
                self.build_history.redo_action(map)
            }
            // Clear/cut/copy/paste will be handled later
            _ => {Ok(())}, 
        }
    }

    // This is our chance to edit the map outside of an mouse handler...
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
        }
    }
}
