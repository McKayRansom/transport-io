use crate::{
    grid::{Direction, Position, Rectangle, Tile},
    map::Map, tileset::Tileset,
};
use macroquad::{
    color::Color,
    input::{get_char_pressed, is_mouse_button_down, mouse_position, mouse_wheel, MouseButton},
    math::vec2,
    ui::{
        hash, root_ui,
        widgets::{self},
        Ui,
    },
    window::{screen_height, screen_width},
};
use macroquad_profiler::ProfilerParams;

const SELECTED_BUILD: Color = Color::new(0., 1.0, 0., 0.3);
const SELECTED_DELETE: Color = Color::new(1.0, 0., 0., 0.3);

const WASD_MOVE_SENSITIVITY: f32 = 10.;
const SCROLL_SENSITIVITY: f32 = 0.9;
const PLUS_MINUS_SENSITVITY: f32 = 0.8;

#[derive(Clone, Copy, PartialEq)]
enum BuildMode {
    None,
    // Vehicle,
    // Station,
    AddRoad,
    RemoveRoad,
    Clear,
    Yield,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ToolbarItem {
    build_mode: BuildMode,
    label: &'static str,
}

// #[derive(Clone, Copy)]
pub struct UiState {
    pub request_quit: bool,
    pub paused: bool,
    pub zoom: f32,
    pub camera: (f32, f32),
    mouse_pressed: bool,
    last_mouse_pos: Position,
    build_mode: BuildMode,
    toolbar_items: Vec<ToolbarItem>,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            request_quit: false,
            paused: false,
            zoom: 1.,
            camera: (0., 0.),
            mouse_pressed: false,
            last_mouse_pos: Position::new(0, 0),
            build_mode: BuildMode::None,
            toolbar_items: vec![
                ToolbarItem {
                    build_mode: BuildMode::AddRoad,
                    label: "Road+",
                },
                ToolbarItem {
                    build_mode: BuildMode::RemoveRoad,
                    label: "Road-",
                },
                ToolbarItem {
                    build_mode: BuildMode::Clear,
                    label: "Delete",
                },
                ToolbarItem {
                    build_mode: BuildMode::Yield,
                    label: "Yield",
                },
            ],
        }
    }

    pub fn update(&mut self, map: &mut Map) {
        while let Some(key) = get_char_pressed() {
            println!("Keydown: {key:?}");
            // TODO: Deal with repeat
            self.key_down_event(key);
        }

        let new_mouse_wheel = mouse_wheel();
        let new_mouse_pos = mouse_position();

        if root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1)) {
            return;
        }

        if new_mouse_wheel.1 != 0. {
            if new_mouse_wheel.1 > 0. {
                self.zoom *= /*new_mouse_wheel.1 * */SCROLL_SENSITIVITY;
            } else {
                self.zoom /= /*(-new_mouse_wheel.1) * */SCROLL_SENSITIVITY;
            }

            println!("Zoom + {} = {}", new_mouse_wheel.1, self.zoom);
        }

        let pos = Position::from_screen(new_mouse_pos, self.camera, self.zoom);

        if is_mouse_button_down(MouseButton::Left) {
            // macroquad::ui::
            if !self.mouse_pressed {
                self.mouse_button_down_event(pos, map)
            }
            self.mouse_pressed = true;
        } else {
            self.mouse_pressed = false;
        }

        if self.last_mouse_pos != pos {
            self.mouse_motion_event(pos, map);
            self.last_mouse_pos = pos;
        }
    }

    fn draw_toolbar(&self) -> BuildMode {
        let toolbar_item_count: f32 = 5.;
        let toolbar_item_width: f32 = 64.;
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
            let mut position = vec2(0., 0.);
            for toolbar_item in &self.toolbar_items {
                let tag = if self.build_mode == toolbar_item.build_mode {
                    "*"
                } else {
                    ""
                };
                if ui.button(position, format!("{}{}{}", tag, toolbar_item.label, tag)) {
                    build_mode = BuildMode::AddRoad;
                }
                position.x += toolbar_item_width + toolbar_item_pad;
            }
        });

        build_mode
    }

    fn draw_tile_details(&self, ui: &mut Ui, map: &Map) {
        if let Some(tile) = map.path_grid.get_tile(&self.last_mouse_pos) {
            match tile {
                Tile::Empty => {
                    ui.label(None, &format!("Empty"));
                }
                Tile::House(house) => {
                    ui.label(None, &format!("House {:?}", house.people_heading_to));
                }
                Tile::Road(road) => {
                    ui.label(None, &format!("Road {:?}", road.reserved));
                }
            }
        }
    }

    fn draw_details(&self, map: &Map) {
        let details_height = 100.;
        let details_width = 200.;
        widgets::Window::new(
            hash!(),
            vec2(
                screen_width() - details_width,
                screen_height() - details_height,
            ),
            vec2(details_width, details_height),
        )
        .label("Details")
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            self.draw_tile_details(ui, map);
        });
    }

    fn draw_paused(&mut self) {
        let paused_height = 50.;
        let paused_width = 75.;
        widgets::Window::new(
            hash!(),
            vec2(screen_width() - paused_width, 0.),
            vec2(paused_width, paused_height),
        )
        .label("Time")
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            let label = if self.paused { "**play**" } else { "pause" };

            if ui.button(None, label) {
                self.paused = !self.paused;
            }
        });
    }

    pub fn draw(&mut self, delivered: u32, map: &Map, tileset: &Tileset) {
        // Score
        widgets::Window::new(hash!(), vec2(0.0, 0.0), vec2(100., 50.))
            .label("Score")
            .movable(false)
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, &format!("Delivered: {}", delivered));
            });

        self.build_mode = self.draw_toolbar();

        self.draw_details(map);

        self.draw_paused();

        // draw selected
        let color = if self.build_mode == BuildMode::Clear {
            SELECTED_DELETE
        } else {
            SELECTED_BUILD
        };

        tileset.draw_rect(&Rectangle::from_pos(self.last_mouse_pos), color);

        // profiler
        macroquad_profiler::profiler(ProfilerParams{fps_counter_pos: vec2(0., 50.)});
    }

    fn key_down_event(&mut self, ch: char) {
        if ch >= '1' && ch < '9' {
            let toolbar_count: usize = ch as usize - '1' as usize;
            if toolbar_count < self.toolbar_items.len() {
                self.build_mode = self.toolbar_items[toolbar_count].build_mode;
            }
            return;
        }
        match ch {
            'q' => self.request_quit = true,
            ' ' => self.paused = !self.paused,
            'w' => self.camera.1 -= WASD_MOVE_SENSITIVITY,
            'a' => self.camera.0 -= WASD_MOVE_SENSITIVITY,
            's' => self.camera.1 += WASD_MOVE_SENSITIVITY,
            'd' => self.camera.0 += WASD_MOVE_SENSITIVITY,
            '-' => self.zoom *= PLUS_MINUS_SENSITVITY,
            '=' => self.zoom /= PLUS_MINUS_SENSITVITY,

            _ => {} // }
        }
    }

    fn mouse_button_down_event(&mut self, pos: Position, map: &mut Map) {
        println!("Mouse pressed: pos: {pos:?}");
        match self.build_mode {
            BuildMode::Clear => {
                map.path_grid.clear_tile(&pos);
            }
            BuildMode::Yield => {
                if let Some(Tile::Road(road)) = map.path_grid.get_tile_mut(&pos) {
                    road.should_yield = !road.should_yield;
                }
            }
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        if is_mouse_button_down(MouseButton::Left) {
            match self.build_mode {
                BuildMode::AddRoad => {
                    let dir = Direction::from_position(self.last_mouse_pos, pos);
                    map.path_grid.add_tile_connection(&self.last_mouse_pos, dir);
                }
                BuildMode::RemoveRoad => {
                    let dir = Direction::from_position(self.last_mouse_pos, pos);
                    map.path_grid
                        .remove_tile_connection(&self.last_mouse_pos, dir);
                }
                BuildMode::Clear => {
                    map.path_grid.clear_tile(&pos);
                }
                _ => {}
            }
        }
        println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);
    }
}
