use crate::{
    grid::{Position, GRID_CELL_SIZE},
    map::{Map, GRID_CENTER},
    menu::{self, MenuSelect},
    tile::Tile,
    tileset::{Sprite, Tileset},
    vehicle::Vehicle,
};
use grades::Grades;
use macroquad::{
    input::{
        get_char_pressed, is_key_down, is_mouse_button_down, mouse_position, mouse_wheel, KeyCode,
        MouseButton,
    },
    math::vec2,
    ui::{
        hash, root_ui,
        widgets::{self},
        Ui,
    },
    window::{screen_height, screen_width},
};
use macroquad_profiler::ProfilerParams;
use toolbar::{Toolbar, ToolbarItem, ToolbarType};
use view_build::ViewBuild;

mod grades;
mod toolbar;
mod view_build;
mod skin;

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.1;
const PLUS_MINUS_SENSITVITY: f32 = 0.8;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 4.;

enum ViewMode {
    Build,
    Route,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UiMenuStatus {
    MainMenu,
    InGame,
    MenuOpen,
}

pub struct UiState {
    draw_profiler: bool,
    pub request_quit: bool,
    pub paused: bool,
    pub zoom: f32,
    pub camera: (f32, f32),
    mouse_pressed: bool,
    last_mouse_pos: Option<Position>,
    view_toolbar: Toolbar<ViewMode>,
    view_build: ViewBuild,
    grades: Grades,
    menu_status: UiMenuStatus,
}

impl UiState {
    pub async fn new() -> Self {
        skin::init().await;

        UiState {
            draw_profiler: false,
            request_quit: false,
            paused: false,
            zoom: 1.,
            camera: (
                GRID_CENTER.0 as f32 * GRID_CELL_SIZE.0 - screen_width() / 2.,
                GRID_CENTER.1 as f32 * GRID_CELL_SIZE.1 - screen_height() / 2.,
            ),
            mouse_pressed: false,
            last_mouse_pos: None,
            view_toolbar: Toolbar::new(
                ToolbarType::Veritcal,
                vec![
                    ToolbarItem::new(ViewMode::Build, "Build stuff", 'b', Sprite::new(9, 0)),
                    ToolbarItem::new(ViewMode::Route, "Route stuff", 'r', Sprite::new(9, 1)),
                ],
            ),
            view_build: ViewBuild::new(),
            grades: Grades::new().await,
            menu_status: UiMenuStatus::MainMenu,
        }
    }

    pub fn update(&mut self, map: &mut Map) {

        while let Some(key) = get_char_pressed() {
            println!("Keydown: {key:?}");
            // TODO: Deal with repeat
            self.key_down_event(key);
        }

        // check WASD
        if is_key_down(KeyCode::W) {
            self.camera.1 -= WASD_MOVE_SENSITIVITY / self.zoom;
        }
        if is_key_down(KeyCode::A) {
            self.camera.0 -= WASD_MOVE_SENSITIVITY / self.zoom;
        }
        if is_key_down(KeyCode::S) {
            self.camera.1 += WASD_MOVE_SENSITIVITY / self.zoom;
        }
        if is_key_down(KeyCode::D) {
            self.camera.0 += WASD_MOVE_SENSITIVITY / self.zoom;
        }

        let new_mouse_wheel = mouse_wheel();
        let new_mouse_pos = mouse_position();

        if root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1)) {
            return;
        }

        if new_mouse_wheel.1 != 0. {
            self.change_zoom(SCROLL_SENSITIVITY * new_mouse_wheel.1);
            println!("Zoom + {} = {}", new_mouse_wheel.1, self.zoom);
        }

        if self.view_build.is_mouse_over(new_mouse_pos) {
            return;
        }

        let pos = Position::from_screen(new_mouse_pos, self.camera, self.zoom);
        {
            if is_mouse_button_down(MouseButton::Left) {
                // macroquad::ui::
                if !self.mouse_pressed {
                    self.view_build.mouse_button_down_event(pos, map)
                }
                self.mouse_pressed = true;
            } else {
                self.mouse_pressed = false;
            }

            if self
                .last_mouse_pos
                .is_none_or(|last_moust_pos| last_moust_pos != pos)
            {
                self.view_build.mouse_motion_event(pos, map);
                self.last_mouse_pos = Some(pos);
            }
        }

        self.view_build.update();
    }

    fn draw_vehicle_details(&self, ui: &mut Ui, tileset: &Tileset, vehicle: &Vehicle) {
        ui.label(
            None,
            &format!("Vehicle Trip: {:?}", vehicle.trip_completed_percent()),
        );
        // self.grades.draw(ui, vehicle.trip_completed_percent());

        ui.label(None, &format!("Vehicle Late: {:?}", vehicle.trip_late()));
        self.grades.draw(ui, vehicle.trip_late());
        vehicle.draw_detail(tileset);
    }

    fn draw_tile_details(
        &self,
        pos: Position,
        ui: &mut Ui,
        map: &Map,
        tileset: &Tileset,
    ) -> Option<()> {
        match map.grid.get_tile(&pos)? {
            Tile::Empty => {
                ui.label(None, "Empty");
            }
            Tile::Ramp(_) => {
                ui.label(None, "Ramp");
            }
            Tile::Building(buliding_id) => {
                if let Some(building) = map.buildings.hash_map.get(buliding_id) {
                    ui.label(None, &format!("Building {:?}", building.vehicle_on_the_way));
                    if let Some(vehicle_id) = building.vehicle_on_the_way {
                        if let Some(vehicle) = map.vehicles.hash_map.get(&vehicle_id) {
                            // vehicle.draw_detail(tileset);
                            self.draw_vehicle_details(ui, tileset, vehicle);
                        }
                    }
                }
            }
            Tile::Road(road) => {
                ui.label(None, &format!("Road {:?}", road));
                if let Some(vehicle_id) = road.reserved.get_reserved_id() {
                    if let Some(vehicle) = map.vehicles.hash_map.get(&vehicle_id) {
                        self.draw_vehicle_details(ui, tileset, vehicle);
                    }
                }
            }
        }

        Some(())
    }

    fn draw_details(&self, map: &Map, tileset: &Tileset) {
        let details_height = 200.;
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
        .ui(&mut root_ui(), |ui| {
            if let Some(pos) = self.last_mouse_pos {
                self.draw_tile_details(pos, ui, map, tileset);
            }
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
        .ui(&mut root_ui(), |ui| {
            let label = if self.paused { "**play**" } else { "pause" };

            if ui.button(None, label) {
                self.paused = !self.paused;
            }
        });
    }

    pub fn draw(&mut self, map: &Map, tileset: &Tileset) -> MenuSelect {
        // Score
        match self.menu_status {
            UiMenuStatus::InGame => {
                self.draw_details(map, tileset);

                self.draw_paused();

                // profiler
                if self.draw_profiler {
                    macroquad_profiler::profiler(ProfilerParams {
                        fps_counter_pos: vec2(0., 0.),
                    });
                }
                self.view_build.draw(map, tileset);

                self.view_toolbar.draw(tileset, 0., screen_height() / 2.0);

                MenuSelect::None
            }
            UiMenuStatus::MainMenu => {
                let status = menu::draw();
                if status != MenuSelect::None {
                    self.menu_status = UiMenuStatus::InGame;
                }
                status
            }

            UiMenuStatus::MenuOpen => {
                let status = menu::draw();
                if status != MenuSelect::None {
                    self.menu_status = UiMenuStatus::InGame;
                }
                status
            }
        }
    }

    fn change_zoom(&mut self, amount: f32) {
        let new_zoom = self.zoom + amount;

        if new_zoom <= MIN_ZOOM || new_zoom >= MAX_ZOOM {
            return;
        }

        let old_screen_zoom = 1. / self.zoom;
        let new_screen_zoom = 1. / new_zoom;
        self.camera.0 += screen_width() * (old_screen_zoom - new_screen_zoom) / 2.;
        self.camera.1 += screen_height() * (old_screen_zoom - new_screen_zoom) / 2.;

        self.zoom += amount;
        // let self.zoom = self.zoom.round();
    }

    fn key_down_event(&mut self, ch: char) {
        match ch {
            'q' => self.request_quit = true,
            ' ' => self.paused = !self.paused,
            'p' => self.draw_profiler = !self.draw_profiler,

            '-' => self.zoom *= PLUS_MINUS_SENSITVITY,
            '=' => self.zoom /= PLUS_MINUS_SENSITVITY,

            '\u{1b}' => {
                if self.menu_status == UiMenuStatus::InGame {
                    self.menu_status = UiMenuStatus::MenuOpen;
                } else {
                    self.menu_status = UiMenuStatus::InGame;
                }
            }

            _ => {
                self.view_build.key_down(ch);
            } // }
        }
    }
}
