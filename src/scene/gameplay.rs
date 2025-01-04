use std::path::Path;

// use super::pause::Pause;
use super::Scene;
// use crate::audio::play_sfx;
use crate::context::Context;
// use crate::input::action_pressed;
// use crate::input::Action;
use crate::map::Map;
use crate::menu::MenuSelect;
// use crate::text::draw_text;
use crate::ui::UiState;
use macroquad::time::get_time;

pub struct Gameplay {
    menu: bool,
    map: Map,
    ui: UiState,
    last_ui_update: f64,
    last_map_update: f64,
}

impl Gameplay {
    pub async fn new() -> Self {
        Gameplay {
            menu: true,
            map: Map::new(),
            ui: UiState::new().await,
            last_ui_update: get_time(),
            last_map_update: get_time(),
        }
    }
}

impl Scene for Gameplay {
    fn update(&mut self, ctx: &mut Context) {
        let speed = 1. / 60.;
        let map_speed = 1. / 16.;

        if get_time() - self.last_ui_update > speed {
            self.ui.update(&mut self.map);
            self.last_ui_update = get_time();

            ctx.tileset.zoom = self.ui.zoom;
            ctx.tileset.camera = self.ui.camera;
            ctx.request_quit = self.ui.request_quit;
        }

        if !self.ui.paused && get_time() - self.last_map_update > map_speed {
            self.map.update();
            self.last_map_update = get_time();
        }
        
    }

    fn draw(&mut self, ctx: &mut Context) {
        self.map.draw(&ctx.tileset);

        match self.ui.draw(&self.map, &ctx.tileset) {
            MenuSelect::Continue => {
                if let Ok(map) = Map::load_from_file(Path::new("saves/game.json")) {
                    self.map = map;
                    self.menu = false;
                }
            }

            MenuSelect::NewGame => {
                self.map = Map::new();
                if self.map.generate().is_err() {
                    println!("Error generating map!");
                }
                self.menu = false;
            }

            MenuSelect::Save => {
                self.map.save_to_file(Path::new("saves/game.json")).unwrap();
            }

            _ => {}
        }
    }
}

