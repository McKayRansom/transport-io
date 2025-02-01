use crate::{
    context::Context,
    map::{
        draw::draw_vehicle_detail, levels::new_level, tile::Tile, vehicle::{Vehicle, SPEED_TICKS}, Map, Position, Unlocked,
    },
    tileset::{Sprite, Tileset},
};
use macroquad::{
    input::{
        get_char_pressed, is_key_down, is_mouse_button_down, mouse_position, mouse_wheel, KeyCode,
        MouseButton,
    },
    math::vec2,
    ui::{
        hash, root_ui,
    },
    window::{screen_height, screen_width},
};
use macroquad_profiler::ProfilerParams;
use menu::{Menu, MenuItem};
use toolbar::{Toolbar, ToolbarItem, ToolbarType, TOOLBAR_SPACE};
use view_build::ViewBuild;

mod build_history;
pub mod menu;
pub mod popup;
pub mod skin;
mod toolbar;
mod view_build;

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.1;
const PLUS_MINUS_SENSITVITY: f32 = 0.8;

#[derive(PartialEq, Eq)]
pub enum TimeSelect {
    Pause,
    FastForward,
    Menu,
}

enum ViewMode {
    Build,
    Route,
}

enum PauseMenuSelect {
    Continue,
    Save,
    Quit,
    Restart,
}

pub struct UiState {
    draw_profiler: bool,
    mouse_pressed: bool,
    last_mouse_pos: Option<Position>,
    pub time_select: Toolbar<TimeSelect>,
    view_toolbar: Toolbar<ViewMode>,
    view_build: ViewBuild,
    pause_menu: Menu<PauseMenuSelect>,
}

impl UiState {
    pub async fn new(unlocked: Unlocked) -> Self {
        UiState {
            draw_profiler: false,

            mouse_pressed: false,
            last_mouse_pos: None,
            time_select: Toolbar::new(
                ToolbarType::Horizontal,
                vec![
                    ToolbarItem::new(TimeSelect::Pause, "Pause game", ' ', Sprite::new(10, 0)),
                    // ToolbarItem::new(TimeSelect::Play, "Play game", ' ', Sprite::new(10, 1)),
                    ToolbarItem::new(
                        TimeSelect::FastForward,
                        "Fast Forward game",
                        ' ',
                        Sprite::new(10, 2),
                    ),
                    ToolbarItem::new(
                        TimeSelect::Menu,
                        "Pause Menu",
                        '\u{1b}',
                        Sprite::new(10, 3),
                    )
                ],
            ),
            view_toolbar: Toolbar::new(
                ToolbarType::Veritcal,
                vec![
                    ToolbarItem::new(ViewMode::Build, "Build stuff", 'b', Sprite::new(9, 0)),
                    ToolbarItem::new(ViewMode::Route, "Route stuff", 'r', Sprite::new(9, 1)),
                ],
            ),
            view_build: ViewBuild::new(unlocked),
            pause_menu: Menu::new(vec![
                MenuItem::new(PauseMenuSelect::Continue, "Close".to_string()),
                MenuItem::new(PauseMenuSelect::Save, "Save".to_string()),
                MenuItem::new(PauseMenuSelect::Quit, "Menu".to_string()),
                MenuItem::new(PauseMenuSelect::Restart, "Restart".to_string()),
            ]),
        }
    }

    fn update_mouse(&mut self, ctx: &mut Context, map: &mut Map) {
        let new_mouse_wheel = mouse_wheel();
        let new_mouse_pos = mouse_position();

        if root_ui().is_mouse_over(vec2(new_mouse_pos.0, new_mouse_pos.1)) {
            return;
        }

        if new_mouse_wheel.1 != 0. {
            ctx.tileset
                .change_zoom(SCROLL_SENSITIVITY * new_mouse_wheel.1);
            println!("Zoom + {} = {}", new_mouse_wheel.1, ctx.tileset.zoom);
        }

        self.view_build.update(map);

        if self.view_build.is_mouse_over(new_mouse_pos) || self.time_select.is_mouse_over(new_mouse_pos) {
            return;
        }

        let pos = Position::from_screen(new_mouse_pos, ctx.tileset.camera, ctx.tileset.zoom);
        {
            if map.grid.get_tile(&pos).is_none() {
                self.view_build.mouse_clear();
                self.last_mouse_pos = None;
                self.mouse_pressed = false;
                return;
            }

            if is_mouse_button_down(MouseButton::Left) {
                // macroquad::ui::
                if !self.mouse_pressed {
                    self.view_build.mouse_button_down_event(pos, map);
                    self.mouse_pressed = true;
                }
            } else if self.mouse_pressed {
                self.mouse_pressed = false;
                self.view_build.mouse_button_up_event(pos, map);
            }

            if self
                .last_mouse_pos
                .is_none_or(|last_moust_pos| last_moust_pos != pos)
            {
                self.view_build.mouse_motion_event(pos, map);
                self.last_mouse_pos = Some(pos);
            }
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

        self.update_mouse(ctx, map);
    }

    fn draw_vehicle_details(&self, map: &Map, tileset: &Tileset, vehicle: &Vehicle) {
        // ui.label(None, &format!("VT: {:?}", vehicle.path.trip_completed_percent()));
        // self.grades.draw(ui, vehicle.trip_completed_percent());

        // ui.label(None, &format!("VL: {:?}", vehicle.path.trip_late()));
        // self.grades.draw(ui, vehicle.trip_late());
        draw_vehicle_detail(map, vehicle, tileset);
    }

    fn draw_tile_details(
        &self,
        pos: Position,
        // ui: &mut Ui,
        map: &Map,
        ctx: &Context,
    ) -> Option<()> {
        match map.grid.get_tile(&pos)? {
            Tile::Empty => {
                // ui.label(None, "Empty");
            }
            Tile::Ramp(_) => {
                // ui.label(None, "Ramp");
            }
            Tile::Water => {
                // ui.label(None, "Water");
            }
            Tile::Building(building) => {
                if let Some(building) = map.get_building(building) {
                    // ui.label(None, &format!("Building {:?}", building.vehicle_on_the_way));
                    if let Some(vehicle_id) = building.vehicle_on_the_way {
                        if let Some(vehicle) = map.vehicles.hash_map.get(&vehicle_id) {
                            // vehicle.draw_detail(tileset);
                            self.draw_vehicle_details(map, &ctx.tileset, vehicle);
                        }
                    }
                }
            }
            Tile::Road(road) => {
                // ui.label(None, &format!("Road {:?}", road));
                if let Some(vehicle_id) = road.reserved.get_reserved_id(map.tick, map.tick + SPEED_TICKS /2) {
                    if let Some(vehicle) = map.vehicles.hash_map.get(&vehicle_id) {
                        self.draw_vehicle_details(map, &ctx.tileset, vehicle);
                    }
                }
                // if let Some(station_id) = road.station {
                    // ui.label(None, &format!("S {:?}", station_id));
                    // println!("Station")
                // }
            }
        }

        Some(())
    }

    pub fn draw(&mut self, map: &Map, ctx: &mut Context) {

        // profiler
        if self.draw_profiler {
            macroquad_profiler::profiler(ProfilerParams {
                fps_counter_pos: vec2(0., 0.),
            });
            self.view_toolbar.draw(ctx, 0., screen_height() / 2.0);
        } 
        if let Some(pos) = self.last_mouse_pos {
            self.draw_tile_details(pos, map, ctx);
        }

        self.time_select.items[0].sprite =
            if self.time_select.get_selected() == Some(&TimeSelect::Pause) {
                Sprite::new(10, 1)
            } else {
                Sprite::new(10, 0)
            };
        self.time_select
            .draw(ctx, screen_width() - TOOLBAR_SPACE * 1.5, 0.);
        self.view_build.draw(map, ctx);

        if self.time_select.get_selected() == Some(&TimeSelect::Menu) {
            if let Some(selected) = self.pause_menu.draw(hash!()) {
                match selected {
                    PauseMenuSelect::Continue => {
                        self.time_select.clear_selected();
                    }
                    PauseMenuSelect::Save => {
                        map.save().expect("Failed to save!");
                    }
                    PauseMenuSelect::Quit => {
                        ctx.switch_scene_to = Some(crate::scene::EScene::MainMenu)
                    }
                    PauseMenuSelect::Restart => {
                        ctx.switch_scene_to = Some(crate::EScene::Gameplay(Box::new(new_level(
                            map.metadata.level_number,
                        ))))
                    }
                }
            }
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, ch: char) {
        match ch {
            'q' => ctx.request_quit = true,
            // ' ' => self.paused = !self.paused,
            'p' => self.draw_profiler = !self.draw_profiler,

            '-' => ctx.tileset.zoom *= PLUS_MINUS_SENSITVITY,
            '=' => ctx.tileset.zoom /= PLUS_MINUS_SENSITVITY,

            _ => {
                self.time_select.key_down(ch);
                self.view_build.key_down(ch);
            } // }
        }
    }
}
