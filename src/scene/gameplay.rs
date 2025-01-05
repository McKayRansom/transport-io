// use std::path::Path;

// use super::pause::Pause;
use super::{GameOptions, Scene};
// use crate::audio::play_sfx;
use crate::context::Context;
// use crate::input::action_pressed;
// use crate::input::Action;
use crate::map::Map;
// use crate::menu::MenuSelect;
// use crate::text::draw_text;
use crate::ui::{TimeSelect, UiState};
use macroquad::time::get_time;

pub struct Gameplay {
    map: Map,
    ui: UiState,
    last_ui_update: f64,
    last_map_update: f64,
}

impl Gameplay {
    pub async fn new(options: GameOptions) -> Self {
        Gameplay {
            map: match options {
                GameOptions::New => Map::new(),
                GameOptions::Level(level) => Map::new_level(level),
                // TODO: Handle error!
                GameOptions::Load(path) => Map::load_from_file(path).expect("Failed to load save!"),
            },
            ui: UiState::new().await,
            last_ui_update: get_time(),
            last_map_update: get_time(),
        }
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
            self.map.update();
            self.last_map_update = get_time();
        }
        
    }

    fn draw(&mut self, ctx: &mut Context) {
        self.map.draw(&ctx.tileset);

        self.ui.draw(&self.map, ctx);
        // match self.ui.draw(&self.map, ctx) {
        //     MenuSelect::Continue => {
        //         if let Ok(map) = Map::load_from_file(Path::new("saves/game.json")) {
        //             self.map = map;
        //             self.menu = false;
        //         }
        //     }

        //     MenuSelect::NewGame => {
        //         self.map = Map::new();
        //         if self.map.generate().is_err() {
        //             println!("Error generating map!");
        //         }
        //         self.menu = false;
        //     }

        //     MenuSelect::Save => {
        //         self.map.save_to_file(Path::new("saves/game.json")).unwrap();
        //     }

        //     _ => {}
        // }
    }
}

