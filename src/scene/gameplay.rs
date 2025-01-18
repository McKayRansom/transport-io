use super::{EScene, GameOptions, Scene};
// use crate::audio::play_sfx;
use crate::context::Context;
use crate::map::draw::draw_map;
use crate::map::levels::{new_level, LEVEL_COUNT};
// use crate::input::action_pressed;
// use crate::input::Action;
use crate::map::Map;
use crate::save::LoadResult;
use crate::ui::popup::{Popup, PopupResult};
// use crate::menu::MenuSelect;
// use crate::text::draw_text;
use crate::ui::{TimeSelect, UiState};
use macroquad::time::get_time;

pub const DEFAULT_MAP_SIZE: (i16, i16) = (64, 64);

pub struct Gameplay {
    map: Box<Map>,
    ui: UiState,
    last_ui_update: f64,
    last_map_update: f64,
    popup: Option<Popup>,
}

impl GameOptions {
    pub fn create(&self) -> LoadResult {
        match &self {
            GameOptions::New => Ok(Map::new_generate(DEFAULT_MAP_SIZE)),
            GameOptions::Level(level) => Ok(new_level(*level)),
            GameOptions::Continue => Map::load(),
        }
    }
}

impl Gameplay {
    pub async fn new(ctx: &mut Context, map: Box<Map>) -> Self {
        let gameplay = Gameplay {
            map,
            ui: UiState::new().await,
            last_ui_update: get_time(),
            last_map_update: get_time(),
            popup: None,
        };

        ctx.tileset.reset_camera(gameplay.map.grid.size_px());

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

        if (time_select != Some(&TimeSelect::Pause) && !self.ui.pause_menu_open)
            && get_time() - self.last_map_update > map_speed
        {
            if self.map.update() {
                self.popup = Some(Popup::new(format!(
                    "Level {} completed!",
                    self.map.metadata.level_number
                )));
            }
            self.last_map_update = get_time();
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        draw_map(&self.map, &ctx.tileset);

        self.ui.draw(&self.map, ctx);

        if let Some(popup) = &self.popup {
            match popup.draw() {
                Some(PopupResult::Ok) => {
                    let level_number = self.map.metadata.level_number + 1;
                    ctx.switch_scene_to = if level_number < LEVEL_COUNT {
                        Some(EScene::Gameplay(Box::new(new_level(level_number))))
                    } else {
                        Some(EScene::MainMenu)
                    }
                }
                Some(PopupResult::Cancel) => {
                    self.popup = None;
                    self.map.metadata.level_complete = true;
                }
                None => {}
            }
        }
    }
}
