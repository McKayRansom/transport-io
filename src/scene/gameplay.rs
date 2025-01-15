use super::{EScene, GameOptions, Scene};
// use crate::audio::play_sfx;
use crate::context::Context;
// use crate::input::action_pressed;
// use crate::input::Action;
use crate::map::Map;
use crate::save::LoadResult;
use crate::ui::popup::{Popup, PopupResult};
// use crate::menu::MenuSelect;
// use crate::text::draw_text;
use crate::ui::{TimeSelect, UiState};
use macroquad::time::get_time;
use macroquad::window::{screen_height, screen_width};

pub const DEFAULT_MAP_SIZE: (i16, i16) = (64, 64);

pub struct Gameplay {
    map: Map,
    ui: UiState,
    last_ui_update: f64,
    last_map_update: f64,
    popup: Option<Popup>,
}

impl GameOptions {
    pub fn create(&self) -> LoadResult {
        match &self {
            GameOptions::New => Ok(Map::new_generate(DEFAULT_MAP_SIZE)),
            GameOptions::Level(level) => Ok(Map::new_level(*level)),
            GameOptions::Continue => Map::load(),
        }
    }
}

impl Gameplay {
    pub async fn new(ctx: &mut Context, map: Map) -> Self {
        let gameplay = Gameplay {
            map: map,
            ui: UiState::new().await,
            last_ui_update: get_time(),
            last_map_update: get_time(),
            popup: None,
        };

        // TODO: Camera center function
        let size = gameplay.map.grid.size_px();
        ctx.tileset.camera = (size.0 /2. - screen_width() / 2., size.1 / 2. - screen_height() / 2.);
        ctx.tileset.zoom = 1.;

        gameplay
    }
}

impl Scene for Gameplay {
    fn update(&mut self, ctx: &mut Context) {
        let speed = 1. / 60.;

        if get_time() - self.last_ui_update > speed {
            self.ui.update(ctx, &mut self.map);
            self.last_ui_update = get_time();
        }

        let time_select = self.ui.time_select.get_selected();
        let map_speed = if time_select == Some(&TimeSelect::FastForward) {
            1. / 60.
        } else {
            1. / 16.
        };

        if (time_select != Some(&TimeSelect::Pause) && !self.ui.pause_menu_open) && get_time() - self.last_map_update > map_speed {
            if self.map.update() {
                self.popup = Some(Popup::new(format!("Level completed!")));
            }
            self.last_map_update = get_time();
        }
        
    }

    fn draw(&mut self, ctx: &mut Context) {
        self.map.draw(&ctx.tileset);

        self.ui.draw(&self.map, ctx);

        if let Some(popup) = &self.popup {
            match popup.draw() {
                Some(PopupResult::Ok) => ctx.switch_scene_to = Some(EScene::Gameplay(Map::new_level(self.map.metadata.level_number + 1))),
                Some(PopupResult::Cancel) => {
                    self.popup = None;
                    self.map.metadata.level_complete = true;
                },
                None => {},
            }
        }
    }
}

