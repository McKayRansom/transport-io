use crate::{
    grid::{BuildError, BuildResult, Position, GRID_CELL_SIZE},
    map::{Map, GRID_CENTER},
    menu::{self, MenuSelect},
    tile::Tile,
    tileset::{Sprite, Tileset},
    vehicle::Vehicle,
};
use grades::Grades;
use macroquad::{
    color::{Color, RED},
    input::{
        get_char_pressed, is_key_down, is_mouse_button_down, mouse_position, mouse_wheel, KeyCode,
        MouseButton,
    },
    math::{vec2, Rect, RectOffset},
    text::draw_text,
    ui::{
        hash, root_ui,
        widgets::{self},
        Skin, Ui,
    },
    window::{screen_height, screen_width},
};
use macroquad_profiler::ProfilerParams;
use toolbar::{Toolbar, ToolbarItem};

mod grades;
mod toolbar;

const SELECTED_BUILD: Color = Color::new(0., 1.0, 0., 0.3);
const SELECTED_DELETE: Color = Color::new(1.0, 0., 0., 0.3);

const WASD_MOVE_SENSITIVITY: f32 = 20.;
const SCROLL_SENSITIVITY: f32 = 0.1;
const PLUS_MINUS_SENSITVITY: f32 = 0.8;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 4.;

#[derive(Clone, Copy, PartialEq)]
enum BuildMode {
    // Vehicle,
    // Station,
    // AddRoad,
    // RemoveRoad,
    TwoLaneRoad,
    Bridge,
    Clear,
    // Yield,
    // Debug,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UiMenuStatus {
    MainMenu,
    InGame,
    MenuOpen,
}

const BUILD_ERROR_TIME: u32 = 60 * 3;

pub struct BuildErrorMsg {
    pub screen_pos: (f32, f32),
    pub err: BuildError,
    pub time: u32,
}

// #[derive(Clone, Copy)]
pub struct UiState {
    pub request_quit: bool,
    pub paused: bool,
    pub zoom: f32,
    pub camera: (f32, f32),
    mouse_pressed: bool,
    last_mouse_pos: Option<Position>,
    bridge_start_pos: Option<Position>,
    build_toolbar: Toolbar<BuildMode>,
    grades: Grades,
    menu_status: UiMenuStatus,
    build_err: Option<BuildErrorMsg>,
}

impl UiState {
    pub async fn new() -> Self {
        UiState {
            request_quit: false,
            paused: false,
            zoom: 1.,
            camera: (
                GRID_CENTER.0 as f32 * GRID_CELL_SIZE.0 - screen_width() / 2.,
                GRID_CENTER.1 as f32 * GRID_CELL_SIZE.1 - screen_height() / 2.,
            ),
            mouse_pressed: false,
            last_mouse_pos: None,
            bridge_start_pos: None,
            build_toolbar: Toolbar::new(vec![
                ToolbarItem::new(
                    BuildMode::TwoLaneRoad,
                    "Build a two lane road",
                    '1',
                    Sprite::new(8, 0),
                ),
                ToolbarItem::new(BuildMode::Bridge, "Build a bridge", '2', Sprite::new(8, 1)),
                ToolbarItem::new(BuildMode::Clear, "Delete", '3', Sprite::new(8, 2)),
            ]),
            grades: Grades::new().await,
            menu_status: UiMenuStatus::MainMenu,
            build_err: None,
        }
    }

    pub async fn init(&mut self) {
        let skin2 = {
            // let font = load_ttf_font("examples/ui_assets/MinimalPixel v2.ttf")
            //     .await
            //     .unwrap();
            // let label_style = root_ui()
            //     .style_builder()
            //     .with_font(&font)
            //     .unwrap()
            //     .text_color(Color::from_rgba(120, 120, 120, 255))
            //     .font_size(15)
            //     .build();

            let window_color = Color::from_hex(0x585858);

            let window_style = root_ui()
                .style_builder()
                // .background(
                //     Image::from_file_with_format(
                //         include_bytes!("../examples/ui_assets/window_background_2.png"),
                //         None,
                //     )
                //     .unwrap(),
                // )
                .color_inactive(window_color)
                .color_hovered(window_color)
                .color_selected(window_color)
                .color_clicked(window_color)
                .color(window_color)
                // .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .margin(RectOffset::new(5.0, 5.0, 5.0, 0.0))
                .build();

            // let button_style = root_ui()
            //     .style_builder()
            // .background(
            //     Image::from_file_with_format(
            //         include_bytes!("../examples/ui_assets/button_background_2.png"),
            //         None,
            //     )
            //     .unwrap(),
            // )
            // .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            // .background_hovered(
            //     Image::from_file_with_format(
            //         include_bytes!("../examples/ui_assets/button_hovered_background_2.png"),
            //         None,
            //     )
            //     .unwrap(),
            // )
            // .background_clicked(
            //     Image::from_file_with_format(
            //         include_bytes!("../examples/ui_assets/button_clicked_background_2.png"),
            //         None,
            //     )
            //     .unwrap(),
            // )
            // .with_font(&font)
            // .unwrap()
            // .text_color(Color::from_rgba(180, 180, 100, 255))
            // .font_size(40)
            // .build();

            // let checkbox_style = root_ui()
            //     .style_builder()
            //     .background(
            //         Image::from_file_with_format(
            //             include_bytes!("../examples/ui_assets/checkbox_background.png"),
            //             None,
            //         )
            //         .unwrap(),
            //     )
            //     .background_hovered(
            //         Image::from_file_with_format(
            //             include_bytes!("../examples/ui_assets/checkbox_hovered_background.png"),
            //             None,
            //         )
            //         .unwrap(),
            //     )
            //     .background_clicked(
            //         Image::from_file_with_format(
            //             include_bytes!("../examples/ui_assets/checkbox_clicked_background.png"),
            //             None,
            //         )
            //         .unwrap(),
            //     )
            //     .build();

            // let editbox_style = root_ui()
            //     .style_builder()
            //     .background(
            //         Image::from_file_with_format(
            //             include_bytes!("../examples/ui_assets/editbox_background.png"),
            //             None,
            //         )
            //         .unwrap(),
            //     )
            //     .background_margin(RectOffset::new(2., 2., 2., 2.))
            //     .with_font(&font)
            //     .unwrap()
            //     .text_color(Color::from_rgba(120, 120, 120, 255))
            //     .font_size(25)
            //     .build();

            // let combobox_style = root_ui()
            //     .style_builder()
            //     .background(
            //         Image::from_file_with_format(
            //             include_bytes!("../examples/ui_assets/combobox_background.png"),
            //             None,
            //         )
            //         .unwrap(),
            //     )
            //     .background_margin(RectOffset::new(4., 25., 6., 6.))
            //     .with_font(&font)
            //     .unwrap()
            //     .text_color(Color::from_rgba(120, 120, 120, 255))
            //     .color(Color::from_rgba(210, 210, 210, 255))
            //     .font_size(25)
            //     .build();

            Skin {
                window_style,
                // button_style,
                // label_style,
                // checkbox_style,
                // editbox_style,
                // combobox_style,
                ..root_ui().default_skin()
            }
        };

        root_ui().push_skin(&skin2);
    }

    pub fn update_build_err(&mut self) {
        if let Some(build_err) = &mut self.build_err {
            build_err.time += 1;
            if build_err.time > BUILD_ERROR_TIME {
                self.build_err = None;
            }
        }
    }

    pub fn draw_build_err(&self) {
        if let Some(build_err) = &self.build_err {
            draw_text(
                format!("{:?}", build_err.err).as_str(),
                build_err.screen_pos.0,
                build_err.screen_pos.1 - build_err.time as f32,
                24.,
                RED,
            );
        }
    }

    pub fn on_build_err(&mut self, err: BuildError, pos: (f32, f32)) {
        self.build_err = Some(BuildErrorMsg {
            screen_pos: pos,
            err,
            time: 0,
        })
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

        if self.build_toolbar.is_mouse_over(new_mouse_pos) {
            return;
        }

        let pos = Position::from_screen(new_mouse_pos, self.camera, self.zoom);
        {
            if is_mouse_button_down(MouseButton::Left) {
                // macroquad::ui::
                if !self.mouse_pressed {
                    if let Err(err) = self.mouse_button_down_event(pos, map) {
                        self.on_build_err(err, new_mouse_pos);
                    }
                }
                self.mouse_pressed = true;
            } else {
                self.mouse_pressed = false;
            }

            if self
                .last_mouse_pos
                .is_none_or(|last_moust_pos| last_moust_pos != pos)
            {
                if let Err(err) = self.mouse_motion_event(pos, map) {
                    self.on_build_err(err, new_mouse_pos);
                }
                self.last_mouse_pos = Some(pos);
            }
        }

        self.update_build_err();
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

    fn draw_selected(&self, _map: &Map, tileset: &Tileset) {
        // draw selected
        if let Some(last_mouse_pos) = self.last_mouse_pos {
            match self.build_toolbar.get_selected() {
                Some(BuildMode::Bridge) => {
                    if let Some(start_pos) = self.bridge_start_pos {
                        for pos in start_pos.iter_line_to(last_mouse_pos).0 {
                            tileset.draw_rect(&Rect::from(pos), SELECTED_BUILD);
                        }
                    } else {
                        tileset.draw_rect(&Rect::from(last_mouse_pos), SELECTED_BUILD);
                    }
                }
                // Some(BuildMode::Debug) => {
                //     if let Some((path, _cost)) = map.grid.find_road(&last_mouse_pos) {
                //         for pos in path {
                //             tileset.draw_rect(&Rect::from(pos), SELECTED_DELETE);
                //         }
                //     }
                // }
                Some(BuildMode::Clear) => {
                    tileset.draw_rect(&Rect::from(last_mouse_pos), SELECTED_DELETE);
                }
                _ => {
                    tileset.draw_rect(&Rect::from(last_mouse_pos), SELECTED_BUILD);
                }
            }
        }
    }

    pub fn draw(&mut self, map: &Map, tileset: &Tileset) -> MenuSelect {
        // Score
        match self.menu_status {
            UiMenuStatus::InGame => {
                self.draw_details(map, tileset);

                self.draw_paused();

                self.draw_selected(map, tileset);

                self.draw_build_err();

                // profiler
                macroquad_profiler::profiler(ProfilerParams {
                    fps_counter_pos: vec2(0., 50.),
                });

                self.build_toolbar.draw(tileset);

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
                self.build_toolbar.key_down(ch);
            } // }
        }
    }

    fn mouse_button_down_event(&mut self, mouse_pos: Position, map: &mut Map) -> BuildResult {
        println!("Mouse pressed: pos: {mouse_pos:?}");
        let build_mode = self.build_toolbar.get_selected();
        if build_mode.is_none() {
            return Ok(());
        }
        match build_mode.unwrap() {
            BuildMode::Clear => {
                map.clear_tile(&mouse_pos)?;
            }
            // BuildMode::Yield => {
            //     if let Some(Tile::Road(road)) = map.grid.get_tile_mut(&mouse_pos) {
            //         road.should_yield = !road.should_yield;
            //     }
            // }
            BuildMode::Bridge => {
                if let Some(start_pos) = self.bridge_start_pos {
                    self.bridge_start_pos = None;
                    map.grid.build_bridge(start_pos, mouse_pos)?;
                } else {
                    self.bridge_start_pos = Some(mouse_pos);
                }
            }
            // BuildMode::Roundabout => {
            //     let roundabout = map.add_intersection();
            //     for tile: Tile in map.path_grid.get_tiles(pos, 2) {
            //         if let Tile::Road(road) = tile {
            //             road.intersection = roundabout;
            //         }
            //     }
            // }
            _ => {}
        }

        Ok(())
    }

    fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) -> BuildResult {
        if is_mouse_button_down(MouseButton::Left) && self.build_toolbar.get_selected().is_some() {
            match self.build_toolbar.get_selected().unwrap() {
                // BuildMode::AddRoad => {
                //     if let Some(last_mouse_pos) = self.last_mouse_pos {
                //         let dir = last_mouse_pos.direction_to(pos);
                //         map.grid.build_road(&last_mouse_pos, dir)?
                //     }
                // }
                // BuildMode::RemoveRoad => {
                //     if let Some(last_mouse_pos) = self.last_mouse_pos {
                //         let dir = last_mouse_pos.direction_to(pos);
                //         map.grid.remove_road(&last_mouse_pos, dir)?
                //     }
                // }
                BuildMode::TwoLaneRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        let dir = last_mouse_pos.direction_to(pos);
                        map.grid.build_two_way_road(last_mouse_pos, dir)?;
                    }
                }
                BuildMode::Clear => {
                    map.clear_tile(&pos)?;
                }
                _ => {}
            }
        }
        println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);

        Ok(())
    }
}
