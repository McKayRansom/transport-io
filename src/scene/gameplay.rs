use super::{EScene, GameOptions, Scene};
use crate::consts::{GAME_SPEED_FAST, GAME_SPEED_NORMAL};
// use crate::audio::play_sfx;
use crate::context::{Context, GameSpeed};
use crate::map::draw::draw_map;
use crate::map::levels::{new_level, LEVEL_COUNT};
// use crate::input::action_pressed;
// use crate::input::Action;
use crate::map::Map;
use crate::save::LoadResult;
use crate::ui::popup::{Popup, PopupResult};
// use crate::menu::MenuSelect;
// use crate::text::draw_text;
use crate::ui::UiState;
use macroquad::time::get_time;

pub const DEFAULT_MAP_SIZE: (i16, i16) = (127, 127);

pub struct Gameplay {
    map: Box<Map>,
    ui: UiState,
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
        let unlocked = map.metadata.unlocks;
        let gameplay = Gameplay {
            map,
            ui: UiState::new(unlocked),
            last_map_update: get_time(),
            popup: None,
        };

        ctx.tileset.reset_camera(gameplay.map.grid.size_px());

        gameplay
    }
}

impl Scene for Gameplay {
    fn update(&mut self, ctx: &mut Context) {

        let map_speed = match ctx.game_speed {
            GameSpeed::Paused => return,
            GameSpeed::Normal => GAME_SPEED_NORMAL,
            GameSpeed::FastForward => GAME_SPEED_FAST,
        };

        if get_time() - self.last_map_update > map_speed
        {
            if self.map.update() && self.map.metadata.is_level {
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

        self.ui.draw(&mut self.map, ctx);

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
