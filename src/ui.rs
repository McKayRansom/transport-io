use crate::{
    grid::{Direction, Position},
    map::Map,
    GameState,
};
use macroquad::{
    input::{get_char_pressed, is_mouse_button_down, mouse_position, MouseButton},
    math::vec2,
    ui::{
        hash, root_ui,
        widgets::{self},
    },
    window::{screen_height, screen_width},
};

#[derive(Clone, Copy)]
enum BuildMode {
    None,
    Vehicle,
    Station,
    Road,
    Delete,
}

#[derive(Clone, Copy)]
pub struct UiState {
    pub request_quit: bool,
    mouse_pressed: bool,
    mouse_down: bool,
    last_mouse_pos: Position,
    build_mode: BuildMode,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            request_quit: false,
            mouse_pressed: false,
            mouse_down: false,
            last_mouse_pos: Position { x: 0, y: 0 },
            build_mode: BuildMode::None,
        }
    }

    pub fn update(&mut self, map: &mut Map) {
        while let Some(key) = get_char_pressed() {
            println!("Keydown: {key:?}");
            self.key_down_event(key, false);
        }

        let new_mouse_pos = mouse_position();

        if is_mouse_button_down(MouseButton::Left) {
            // macroquad::ui::
            if !self.mouse_pressed
                && !root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1))
            {
                self.mouse_button_down_event(new_mouse_pos.0, new_mouse_pos.1);
            }
            self.mouse_pressed = true;
        } else {
            self.mouse_pressed = false;
        }

        let pos = Position::from_screen(new_mouse_pos.0, new_mouse_pos.1);
        if self.last_mouse_pos != pos
            && !root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1))
        {
            self.mouse_motion_event(pos, map);
            self.last_mouse_pos = pos;
        }
    }

    fn draw_toolbar(self) -> BuildMode {
        let toolbar_item_count: f32 = 5.;
        let toolbar_item_width: f32 = 32.;
        let toolbar_item_pad: f32 = 10.;
        let toolbar_height: f32 = 32.;

        let toolbar_width = (toolbar_item_width + toolbar_item_pad) * toolbar_item_count;

        let mut build_mode = self.build_mode;

        widgets::Window::new(
            hash!(),
            vec2(
                screen_width() / 2.0 - (toolbar_width / 2.),
                screen_height() - toolbar_height,
            ),
            vec2(toolbar_width, toolbar_height),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            if ui.button(vec2(0., 0.), "road") {
                build_mode = BuildMode::Road;
            }

            if ui.button(vec2(toolbar_item_width + toolbar_item_pad, 0.), "del") {
                build_mode = BuildMode::Delete;
            }
            // ui.is_mouse_over(x)
            // widgets::Button::new("Button")
            // .size(vec2(75., 75.))
            // .ui(ui);
            // ui.separator();
            // widgets::Button::new("Button 2")
            // .size(vec2(75., 75.))
            // .ui(ui);
        });

        build_mode
    }

    pub fn draw(&self, game_state: &GameState) -> UiState {
        let mut new_state = *self;
        // Score
        widgets::Window::new(hash!(), vec2(0.0, 0.0), vec2(100., 50.))
            .label("Score")
            .movable(false)
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, &format!("Delivered: {}", game_state.delivered));
            });

        new_state.build_mode = self.draw_toolbar();

        new_state
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
            // 'a' => {
            // self.build_mode = BuildMode::Vehicle;
            // }
            // 's' => {
            // self.build_mode = BuildMode::Station;
            // }
            '2' => {
                self.build_mode = BuildMode::Delete;
            }
            '1' => {
                self.build_mode = BuildMode::Road;
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
            // BuildMode::Vehicle => {
            //     if self.map.path_grid.is_allowed(&pos) && !self.map.path_grid.is_occupied(&pos) {
            //         self.map.vehicles.push(Vehicle::new(pos, &mut self.map.path_grid))
            //     }
            // }
            // BuildMode::Station => {
            //     if !self.map.path_grid.is_allowed(&pos) {
            //         // self.path_grid.add_allowed(pos);
            //         println!("Not allowed here");
            //     } else {
            //         self.map.stations.push(Station::new(pos))
            //     }
            // }
            // BuildMode::Road => {
            //     // if !self.path_grid.is_allowed(pos) {
            //     self.map.path_grid.add_allowed(&pos, self.build_direction);
            //     // }
            // }
            // BuildMode::Delete => {
            //     if self.map.path_grid.is_allowed(&pos) {
            //         self.map.path_grid.remove_allowed(&pos);
            //     }
            // }
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        if is_mouse_button_down(MouseButton::Left) {
            // _ctx: &mut Context,
            match self.build_mode {
                BuildMode::Road => {
                    map.path_grid
                        .add_allowed(&self.last_mouse_pos, Direction::from_position(self.last_mouse_pos, pos));
                }
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
                // BuildMode::Road => {
                //     self.map.path_grid.add_allowed(&pos, self.build_direction);
                // }
                // BuildMode::Delete => {
                //     if self.map.path_grid.is_allowed(&pos) {
                //         self.map.path_grid.remove_allowed(&pos);
                //     }
                // }
                _ => {}
            }
        }
        println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);
    }
}
