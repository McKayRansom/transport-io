mod grid;
mod map;
mod tileset;
mod vehicle;
mod tile;
mod ui;
mod menu;
use std::path::Path;

use menu::MenuSelect;
use ui::UiState;
use map::Map;
use miniquad::window::set_window_size;
use tileset::Tileset;

use macroquad::prelude::*;


struct GameState {
    menu: bool,
    map: Map,
    ui: UiState,
}

impl GameState {
    pub async fn new() -> Self {
        GameState {
            menu: true,
            map: Map::new(),
            ui: UiState::new().await,
        }
    }

    pub fn load_level(&mut self) {
        self.map.generate();
    }

    fn update(&mut self) {
        self.map.update();
    }

    fn draw(&mut self, tileset: &Tileset) {
        clear_background(BLACK);

        self.map.draw(tileset);

        match self.ui.draw(&self.map, tileset) {
            MenuSelect::Continue => {
                if let Ok(map) = Map::load_from_file(Path::new("saves/game.json")) {
                    self.map = map;
                    self.menu = false;
                }
            }

            MenuSelect::NewGame => {
                self.menu = false;
            }

            MenuSelect::Save => {
                self.map.save_to_file(Path::new("saves/game.json")).unwrap();
            }

            _ => {}
        }
    }

}

#[macroquad::main("Transport IO")]
async fn main() {
    // Next we create a new instance of our GameState struct, which implements EventHandler
    let mut state = GameState::new().await;
    let speed = 1. / 8.;

    set_window_size(800, 800);

    // state.key_manager.add_handler(KeyHandler {key: KeyCode::Q, func: game_quit, help: "Q: Quit the game"});

    let tileset_texture = load_texture("resources/tileset.png").await.unwrap();
    tileset_texture.set_filter(FilterMode::Nearest);

    // let tiled_map_json = load_string("resources/map.json").await.unwrap();
    // let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();

    let mut tileset = Tileset::new(tileset_texture, 16);

    
    state.ui.init().await;

    state.load_level();
    // And finally we actually run our game, passing in our context and state.
    // event::run(ctx, events_loop, state)

    let mut last_update = get_time();

    loop {
        state.ui.update(&mut state.map);

        if !state.ui.paused && get_time() - last_update > speed {
            last_update = get_time();

            state.update();
        }

        tileset.zoom = state.ui.zoom;
        tileset.camera = state.ui.camera;
        state.draw(&tileset);

        // TODO: Take quit request confirmation from example
        if state.ui.request_quit {
            break;
        }

        next_frame().await;
    }
}
