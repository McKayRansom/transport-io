use crate::{
    grid::{Direction, Position, Rectangle},
    map::Map,
    GameState,
};
use macroquad::{
    color::Color,
    input::{get_char_pressed, is_mouse_button_down, mouse_position, MouseButton},
    math::vec2,
    miniquad::graphics,
    shapes::draw_rectangle,
    ui::{
        hash, root_ui,
        widgets::{self},
    },
    window::{screen_height, screen_width},
};

const SELECTED_BUILD: Color = Color::new(0., 1.0, 0., 0.3);
const SELECTED_DELETE: Color = Color::new(1.0, 0., 0., 0.3);

#[derive(Clone, Copy, PartialEq)]
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
    last_mouse_pos: Position,
    build_mode: BuildMode,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            request_quit: false,
            mouse_pressed: false,
            last_mouse_pos: Position { x: 0, y: 0 },
            build_mode: BuildMode::None,
        }
    }

    pub fn update(&mut self, map: &mut Map) {
        while let Some(key) = get_char_pressed() {
            println!("Keydown: {key:?}");
            // TODO: Deal with repeat
            self.key_down_event(key);
        }

        let new_mouse_pos = mouse_position();
        let pos = Position::from_screen(new_mouse_pos.0, new_mouse_pos.1);

        if is_mouse_button_down(MouseButton::Left) {
            // macroquad::ui::
            if !self.mouse_pressed
                && !root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1))
            {
                self.mouse_button_down_event(pos, map)
            }
            self.mouse_pressed = true;
        } else {
            self.mouse_pressed = false;
        }

        if self.last_mouse_pos != pos
            && !root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1))
        {
            self.mouse_motion_event(pos, map);
            self.last_mouse_pos = pos;
        }
    }

    fn draw_toolbar(&self) -> BuildMode {
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

    pub fn draw(&mut self, delivered: u32) {
        // Score
        widgets::Window::new(hash!(), vec2(0.0, 0.0), vec2(100., 50.))
            .label("Score")
            .movable(false)
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, &format!("Delivered: {}", delivered));
            });

        self.build_mode = self.draw_toolbar();

        // draw selected
        let color = if self.build_mode == BuildMode::Delete {
            SELECTED_DELETE
        } else {
            SELECTED_BUILD
        };

        Rectangle::from_pos(self.last_mouse_pos).draw(color);
    }

    fn key_down_event(&mut self, ch: char) {
        match ch {
            'q' => {
                self.request_quit = true;
            }
            '1' => {
                self.build_mode = BuildMode::Road;
            }
            '2' => {
                self.build_mode = BuildMode::Delete;
            }
            _ => {} // }
        }
    }

    fn mouse_button_down_event(&mut self, pos: Position, map: &mut Map) {
        println!("Mouse pressed: pos: {pos:?}");
        match self.build_mode {
            BuildMode::Delete => {
                map.path_grid.remove_allowed(&pos);
            }
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        if is_mouse_button_down(MouseButton::Left) {
            match self.build_mode {
                BuildMode::Road => {
                    map.path_grid.add_allowed(
                        &self.last_mouse_pos,
                        Direction::from_position(self.last_mouse_pos, pos),
                    );
                }
                BuildMode::Delete => {
                    map.path_grid.remove_allowed(&pos);
                }
                _ => {}
            }
        }
        println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);
    }
}
