mod grid;
mod map;
mod station;
mod tileset;
mod vehicle;
mod ui;
use ui::UiState;
use map::Map;
use miniquad::window::set_window_size;
use tileset::Tileset;

use macroquad::prelude::*;

const HELP_TEXT: &'static str = "Transport IO v0.0
Q: Quit
A: Add vehicle
S: Build station
D: Delete Road
F: Build Road
R: Rotate
";

struct GameState {
    map: Map,
    ui: UiState,
    delivered: u32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            map: Map::new(),
            ui: UiState::new(),
            delivered: 0,
        }
    }

    pub fn load_level(&mut self) {
        self.map.generate();
    }

    fn update(&mut self) {
        self.delivered += self.map.update();
    }

    fn draw(&mut self, tileset: &Tileset) {
        clear_background(BLACK);

        self.map.draw(tileset);

        self.ui.draw(self);

    }

}

#[macroquad::main("Transport IO")]
async fn main() {
    // Next we create a new instance of our GameState struct, which implements EventHandler
    let mut state = GameState::new();
    let speed = 1. / 8.;

    set_window_size(800, 800);

    // state.key_manager.add_handler(KeyHandler {key: KeyCode::Q, func: game_quit, help: "Q: Quit the game"});

    let tileset_texture = load_texture("resources/tileset.png").await.unwrap();
    tileset_texture.set_filter(FilterMode::Nearest);

    // let tiled_map_json = load_string("resources/map.json").await.unwrap();
    // let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();

    let tileset = Tileset::new(tileset_texture, 16);

    state.load_level();
    // And finally we actually run our game, passing in our context and state.
    // event::run(ctx, events_loop, state)

    let mut last_update = get_time();

    loop {
        state.ui.update(&mut state.map);

        if get_time() - last_update > speed {
            last_update = get_time();

            state.update();
        }

        state.draw(&tileset);

        // TODO: Take quit request confirmation from example
        if state.ui.request_quit {
            break;
        }

        next_frame().await;
    }
}
