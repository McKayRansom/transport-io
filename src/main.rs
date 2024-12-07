mod grid;
use grid::Direction;
use grid::Position;
mod station;
mod vehicle;
mod tileset;
mod map;
use map::Map;
use station::Station;
use tileset::Tileset;
use vehicle::Vehicle;

use macroquad::prelude::*;


const HELP_TEXT: &'static str = "Transport IO v0.0
Q: Quit
A: Add vehicle
S: Build station
D: Delete Road
F: Build Road
R: Rotate
";


enum BuildMode {
    None,
    Vehicle,
    Station,
    Road,
    Delete,
}

struct GameState {
    map: Map,
    mouse_down: bool,
    build_mode: BuildMode,
    build_direction: Direction,
    delivered: u32,
    request_quit: bool,
}

impl GameState {
    pub fn new() -> Self {

        GameState {
            map: Map::new(),

            mouse_down: false,
            build_mode: BuildMode::None,
            build_direction: Direction::Right,
            delivered: 0,
            request_quit: false,
        }
    }

    pub fn load_level(&mut self) {
        self.map.generate();
    }

    fn update(&mut self) {
        for s in self.map.vehicles.iter_mut() {
            self.delivered += s.update(&self.map.stations, &mut self.map.path_grid);
        }
    }


    fn draw(&self, tileset: &Tileset) {
        clear_background(BLACK);

        self.map.draw(tileset);

        let delivered = self.delivered;
        draw_text(
            format!("Delivered: {delivered:?}").as_str(),
            10.,
            32.,
            43.,
            WHITE,
        );

        let direction = self.build_direction;
        draw_text(
            format!("Direction: {direction:?}").as_str(),
            10.,
            32. + 32.,
            43.,
            WHITE,
        );

        draw_multiline_text(HELP_TEXT, 10., 32. + 64., 43., Some(0.75), WHITE);
    }

    fn key_down_event(&mut self, ch: char, repeat: bool) {
        if repeat {
            return;
        }
        // Here we attempt to convert the Keycode into a Direction using the helper
        // we defined earlier.
        // if let Some(keycode) = input.keycode {
        match ch {
            'q' => {
                self.request_quit = true;
                // ctx.request_quit();
            }
            'a' => {
                self.build_mode = BuildMode::Vehicle;
            }
            's' => {
                self.build_mode = BuildMode::Station;
            }
            'd' => {
                self.build_mode = BuildMode::Delete;
            }
            'f' => {
                self.build_mode = BuildMode::Road;
            }
            'r' => {
                self.build_direction = self.build_direction.rotate();
            }
            _ => {} // }
        }

        // Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        // _ctx: &mut Context,
        x: f32,
        y: f32,
    ) {
        self.mouse_down = true;
        let pos = Position::from_screen(x, y);
        println!("Mouse pressed: pos: {pos:?} x: {x}, y: {y}");
        match self.build_mode {
            BuildMode::Vehicle => {
                if self.map.path_grid.is_allowed(&pos) && !self.map.path_grid.is_occupied(&pos) {
                    self.map.vehicles.push(Vehicle::new(pos, &mut self.map.path_grid))
                }
            }
            BuildMode::Station => {
                if !self.map.path_grid.is_allowed(&pos) {
                    // self.path_grid.add_allowed(pos);
                    println!("Not allowed here");
                } else {
                    self.map.stations.push(Station::new(pos))
                }
            }
            BuildMode::Road => {
                // if !self.path_grid.is_allowed(pos) {
                self.map.path_grid.add_allowed(&pos, self.build_direction);
                // }
            }
            BuildMode::Delete => {
                if self.map.path_grid.is_allowed(&pos) {
                    self.map.path_grid.remove_allowed(&pos);
                }
            }
            _ => {}
        }
    }

    fn mouse_motion_event(
        &mut self,
        // _ctx: &mut Context,
        x: f32,
        y: f32,
    ) {
        if is_mouse_button_down(MouseButton::Left) {
            // Mouse coordinates are PHYSICAL coordinates, but here we want logical coordinates.

            // If you simply use the initial coordinate system, then physical and logical
            // coordinates are identical.
            // self.pos_x = x;
            // self.pos_y = y;

            // If you change your screen coordinate system you need to calculate the
            // logical coordinates like this:
            /*
            let screen_rect = graphics::screen_coordinates(_ctx);
            let size = graphics::window(_ctx).inner_size();
            self.pos_x = (x / (size.width  as f32)) * screen_rect.w + screen_rect.x;
            self.pos_y = (y / (size.height as f32)) * screen_rect.h + screen_rect.y;
            */

            let pos = Position::from_screen(x, y);
            match self.build_mode {
                // BuildMode::Vehicle => {
                // if self.path_grid.is_allowed(pos) {
                // self.snakes.push(Vehicle::new(pos, &mut self.path_grid))
                // }
                // }
                // BuildMode::Station => {
                // if !self.path_grid.is_allowed(pos) {
                // }
                // self.stations.push(Station::new(pos))

                // }
                BuildMode::Road => {
                    self.map.path_grid.add_allowed(&pos, self.build_direction);
                }
                BuildMode::Delete => {
                    if self.map.path_grid.is_allowed(&pos) {
                        self.map.path_grid.remove_allowed(&pos);
                    }
                }
                _ => {}
            }
        }
        println!("Mouse motion, x: {x}, y: {y}");
    }
}

#[macroquad::main("Transport IO")]
async fn main() {
    // Next we create a new instance of our GameState struct, which implements EventHandler
    let mut state = GameState::new();
    let speed = 1./8.;

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
    let mut mouse_pressed = false;
    let mut last_mouse_pos = mouse_position();

    loop {
        while let Some(key) = get_char_pressed() {
            println!("Keydown: {key:?}");
            state.key_down_event(key, false);
        }

        let new_mouse_pos = mouse_position();

        if is_mouse_button_down(MouseButton::Left) {
            if !mouse_pressed {
                state.mouse_button_down_event(new_mouse_pos.0, new_mouse_pos.1);
            }
            mouse_pressed = true;
        } else {
            mouse_pressed = false;
        }

        if last_mouse_pos != new_mouse_pos {
            state.mouse_motion_event(new_mouse_pos.0, new_mouse_pos.1);
            last_mouse_pos = new_mouse_pos;
        }

        if get_time() - last_update > speed {
            last_update = get_time();

            state.update();
        }

        state.draw(&tileset);


        // TODO: Take quit request confirmation from example
        if state.request_quit {
            break;
        }

        next_frame().await;
    }
}
