use crate::{
    context::Context,
    map::{tile::Tile, vehicle::Vehicle, Map, Position},
    tileset::{Sprite, Tileset},
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
use menu::{Menu, MenuItem};
use toolbar::{Toolbar, ToolbarItem, ToolbarType, TOOLBAR_SPACE};
use view_build::ViewBuild;

mod grades;
pub mod menu;
pub mod skin;
mod toolbar;
mod view_build;

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.1;
const PLUS_MINUS_SENSITVITY: f32 = 0.8;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 4.;

#[derive(PartialEq, Eq)]
pub enum TimeSelect {
    Pause,
    FastForward,
}

enum ViewMode {
    Build,
    Route,
}

enum PauseMenuSelect {
    Continue,
    Save,
    Load,
    #[cfg(not(target_family = "wasm"))]
    Quit,
}

pub struct UiState {
    draw_profiler: bool,
    mouse_pressed: bool,
    last_mouse_pos: Option<Position>,
    pub time_select: Toolbar<TimeSelect>,
    view_toolbar: Toolbar<ViewMode>,
    view_build: ViewBuild,
    grades: Grades,
    pub pause_menu_open: bool,
    pause_menu: Menu<PauseMenuSelect>,
}

impl UiState {
    pub async fn new() -> Self {
        UiState {
            draw_profiler: false,

            mouse_pressed: false,
            last_mouse_pos: None,
            time_select: Toolbar::new(
                ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(TimeSelect::Pause, "Pause game", ' ', Sprite::new(10, 0)),
                    // ToolbarItem::new(TimeSelect::Play, "Play game", ' ', Sprite::new(10, 1)),
                    ToolbarItem::new(TimeSelect::FastForward, "Fast Forward game", ' ', Sprite::new(10, 2)),
                ]
            ),
            view_toolbar: Toolbar::new(
                ToolbarType::Veritcal,
                vec![
                    ToolbarItem::new(ViewMode::Build, "Build stuff", 'b', Sprite::new(9, 0)),
                    ToolbarItem::new(ViewMode::Route, "Route stuff", 'r', Sprite::new(9, 1)),
                ],
            ),
            view_build: ViewBuild::new(),
            grades: Grades::new().await,
            pause_menu_open: false,
            pause_menu: Menu::new(vec![
                MenuItem::new(PauseMenuSelect::Continue, "Close".to_string()),
                MenuItem::new(PauseMenuSelect::Save, "Save".to_string()),
                MenuItem::new(PauseMenuSelect::Load, "Load".to_string()),
                #[cfg(not(target_family = "wasm"))]
                MenuItem::new(PauseMenuSelect::Quit, "Menu".to_string()),
            ]),
        }
    }

    pub fn update(&mut self, ctx: &mut Context, map: &mut Map) {
        while let Some(key) = get_char_pressed() {
            println!("Keydown: {key:?}");
            // TODO: Deal with repeat
            self.key_down_event(ctx, key);
        }

        // check WASD
        if is_key_down(KeyCode::W) {
            ctx.tileset.camera.1 -= WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }
        if is_key_down(KeyCode::A) {
            ctx.tileset.camera.0 -= WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }
        if is_key_down(KeyCode::S) {
            ctx.tileset.camera.1 += WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }
        if is_key_down(KeyCode::D) {
            ctx.tileset.camera.0 += WASD_MOVE_SENSITIVITY / ctx.tileset.zoom;
        }

        let new_mouse_wheel = mouse_wheel();
        let new_mouse_pos = mouse_position();

        if root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1)) {
            return;
        }

        if new_mouse_wheel.1 != 0. {
            self.change_zoom(ctx, SCROLL_SENSITIVITY * new_mouse_wheel.1);
            println!("Zoom + {} = {}", new_mouse_wheel.1, ctx.tileset.zoom);
        }

        if self.view_build.is_mouse_over(new_mouse_pos) {
            return;
        }

        let pos = Position::from_screen(new_mouse_pos, ctx.tileset.camera, ctx.tileset.zoom);
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
        ctx: &Context,
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
                            self.draw_vehicle_details(ui, &ctx.tileset, vehicle);
                        }
                    }
                }
            }
            Tile::Road(road) => {
                ui.label(None, &format!("Road {:?}", road));
                if let Some(vehicle_id) = road.reserved.get_reserved_id() {
                    if let Some(vehicle) = map.vehicles.hash_map.get(&vehicle_id) {
                        self.draw_vehicle_details(ui, &ctx.tileset, vehicle);
                    }
                }
            }
        }

        Some(())
    }

    fn draw_details(&self, map: &Map, ctx: &Context) {
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
                self.draw_tile_details(pos, ui, map, ctx);
            }
        });
    }

    pub fn draw(&mut self, map: &Map, ctx: &mut Context) {
        self.draw_details(map, ctx);


        // profiler
        if self.draw_profiler {
            macroquad_profiler::profiler(ProfilerParams {
                fps_counter_pos: vec2(0., 0.),
            });
        }

        self.time_select.items[0].sprite = if self.time_select.get_selected() == Some(&TimeSelect::Pause) {
            Sprite::new(10, 0)
        } else {
            Sprite::new(10, 1)
        };
        self.time_select.draw(ctx, screen_width() - TOOLBAR_SPACE * 1.5, 0.);
        self.view_build.draw(map, ctx);
        self.view_toolbar.draw(ctx, 0., screen_height() / 2.0);

        if self.pause_menu_open {
            if let Some(selected) = self.pause_menu.draw() {
                match selected {
                    PauseMenuSelect::Continue => {
                        self.pause_menu_open = false;
                    }
                    PauseMenuSelect::Quit => {
                        ctx.switch_scene_to = Some(crate::scene::EScene::MainMenu)
                    }
                    _ => {}
                }
            }
        }
    }

    fn change_zoom(&mut self, ctx: &mut Context, amount: f32) {
        let new_zoom = ctx.tileset.zoom + amount;

        if new_zoom <= MIN_ZOOM || new_zoom >= MAX_ZOOM {
            return;
        }

        let old_screen_zoom = 1. / ctx.tileset.zoom;
        let new_screen_zoom = 1. / new_zoom;
        ctx.tileset.camera.0 += screen_width() * (old_screen_zoom - new_screen_zoom) / 2.;
        ctx.tileset.camera.1 += screen_height() * (old_screen_zoom - new_screen_zoom) / 2.;

        ctx.tileset.zoom += amount;
        // let self.zoom = self.zoom.round();
    }

    fn key_down_event(&mut self, ctx: &mut Context, ch: char) {
        match ch {
            'q' => ctx.request_quit = true,
            // ' ' => self.paused = !self.paused,
            'p' => self.draw_profiler = !self.draw_profiler,

            '-' => ctx.tileset.zoom *= PLUS_MINUS_SENSITVITY,
            '=' => ctx.tileset.zoom /= PLUS_MINUS_SENSITVITY,

            '\u{1b}' => {
                self.pause_menu_open = !self.pause_menu_open;
            }

            _ => {
                self.time_select.key_down(ch);
                self.view_build.key_down(ch);
            } // }
        }
    }
}
