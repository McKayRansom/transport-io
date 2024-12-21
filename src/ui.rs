
use crate::{
    grid::{Position, GRID_CELL_SIZE}, map::{Map, GRID_CENTER}, tile::Tile, tileset::Tileset, vehicle::Vehicle
};
use macroquad::{
    color::Color, input::{get_char_pressed, is_key_down, is_mouse_button_down, mouse_position, mouse_wheel, KeyCode, MouseButton}, math::{vec2, Rect, RectOffset}, ui::{
        hash, root_ui, widgets::{self}, Skin, Ui
    }, window::{screen_height, screen_width}
};
use macroquad_profiler::ProfilerParams;

const SELECTED_BUILD: Color = Color::new(0., 1.0, 0., 0.3);
const SELECTED_DELETE: Color = Color::new(1.0, 0., 0., 0.3);

const WASD_MOVE_SENSITIVITY: f32 = 10.;
const SCROLL_SENSITIVITY: f32 = 0.05;
const PLUS_MINUS_SENSITVITY: f32 = 0.8;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 4.;

#[derive(Clone, Copy, PartialEq)]
enum BuildMode {
    None,
    // Vehicle,
    // Station,
    AddRoad,
    RemoveRoad,
    TwoLaneRoad,
    Bridge,
    Clear,
    Yield,
    Debug,
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
    last_mouse_pos: Option<Position>,
    bridge_start_pos: Option<Position>,
    build_mode: BuildMode,
    toolbar_items: Vec<ToolbarItem>,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            request_quit: false,
            paused: false,
            zoom: 1.,
            camera: (GRID_CENTER.0 as f32 * GRID_CELL_SIZE.0 - screen_width() / 2., GRID_CENTER.1 as f32 * GRID_CELL_SIZE.1 - screen_height() / 2.),
            mouse_pressed: false,
            last_mouse_pos: None,
            bridge_start_pos: None,
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
                    build_mode: BuildMode::TwoLaneRoad,
                    label: "TwoLane",
                },
                ToolbarItem {
                    build_mode: BuildMode::Bridge,
                    label: "Bridge",
                },
                ToolbarItem {
                    build_mode: BuildMode::Clear,
                    label: "Delete",
                },
                ToolbarItem {
                    build_mode: BuildMode::Yield,
                    label: "Yield",
                },
                ToolbarItem {
                    build_mode: BuildMode::Debug,
                    label: "Debug",
                }
            ],
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
                .margin(RectOffset::new(5.0,5.0, 5.0, 0.0))
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

        if let Some(pos) = Position::from_screen(new_mouse_pos, self.camera, self.zoom, map.grid.size) {

            if is_mouse_button_down(MouseButton::Left) {
                // macroquad::ui::
                if !self.mouse_pressed {
                    self.mouse_button_down_event(pos, map)
                }
                self.mouse_pressed = true;
            } else {
                self.mouse_pressed = false;
            }


            if self.last_mouse_pos.is_none_or(|last_moust_pos| last_moust_pos != pos ) {
                self.mouse_motion_event(pos, map);
                self.last_mouse_pos = Some(pos);
            }

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
        .ui(&mut root_ui(), |ui| {
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

    fn draw_vehicle_details(ui: &mut Ui, tileset: &Tileset, vehicle: &Vehicle) {
        ui.label(None, &format!("Vehicle Trip: {:?}", vehicle.trip_completed_percent()));
        ui.label(None, &format!("Vehicle Late: {:?}", vehicle.trip_late()));
        vehicle.draw_detail(tileset);
    }

    fn draw_tile_details(pos: Position, ui: &mut Ui, map: &Map, tileset: &Tileset) {
        match map.grid.get_tile(&pos) {
            Tile::Empty => {
                ui.label(None, "Empty");
            }
            Tile::House(house) => {
                ui.label(None, &format!("House {:?}", house.vehicle_on_the_way));
                if let Some(vehicle_id) = house.vehicle_on_the_way {
                    if let Some(vehicle) = map.vehicles.get(&vehicle_id) {
                        // vehicle.draw_detail(tileset);
                        UiState::draw_vehicle_details(ui, tileset, vehicle);
                    }
                }
            }
            Tile::Road(road) => {
                ui.label(None, &format!("Road {:?}", road));
                if let Some(vehicle_id) = road.reserved.get_reserved_id() {
                    if let Some(vehicle) = map.vehicles.get(&vehicle_id) {
                        UiState::draw_vehicle_details(ui, tileset, vehicle);
                    }
                }
            }
        }
    }

    fn draw_details(&self, map: &Map, tileset: &Tileset) {
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
        .ui(&mut root_ui(), |ui| {
            if let Some(pos) = self.last_mouse_pos {
                UiState::draw_tile_details(pos, ui, map, tileset);
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

    fn draw_selected(&self, map: &Map, tileset: &Tileset) {
        // draw selected
        if let Some(last_mouse_pos) = self.last_mouse_pos {
            match self.build_mode {
                BuildMode::Bridge => {
                    if let Some(start_pos) = self.bridge_start_pos {
                        for pos in start_pos.iter_line_to(last_mouse_pos, map.grid.size).0 {
                            tileset.draw_rect(&Rect::from(pos), SELECTED_BUILD);
                        }
                    } else {
                        tileset.draw_rect(&Rect::from(last_mouse_pos), SELECTED_BUILD);
                    }
                }
                BuildMode::Debug => {
                    if let Some((path, _cost)) = map.grid.find_road(&last_mouse_pos)
                    {
                        for pos in path {
                            tileset.draw_rect(&Rect::from(pos), SELECTED_DELETE);
                        }

                    }
                }
                _ => {
                    let color = if self.build_mode == BuildMode::Clear {
                        SELECTED_DELETE
                    } else {
                        SELECTED_BUILD
                    };
                    tileset.draw_rect(&Rect::from(last_mouse_pos), color);
                }
            }
        }

    }

    pub fn draw(&mut self, map: &Map, tileset: &Tileset) {
        // Score
        widgets::Window::new(hash!(), vec2(0.0, 0.0), vec2(100., 50.))
            .label("Score")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                ui.label(None, &format!("Rating: {}", map.rating));
            });

        self.build_mode = self.draw_toolbar();

        self.draw_details(map, tileset);

        self.draw_paused();

        self.draw_selected(map, tileset);

        // profiler
        macroquad_profiler::profiler(ProfilerParams{fps_counter_pos: vec2(0., 50.)});
    }

    fn change_zoom(&mut self, amount: f32) {

        let new_zoom = self.zoom + amount;

        if new_zoom <= MIN_ZOOM || new_zoom >= MAX_ZOOM {
            return;
        }

        let old_screen_zoom = 1./self.zoom;
        let new_screen_zoom = 1./new_zoom;
        self.camera.0 += screen_width() * (old_screen_zoom - new_screen_zoom) / 2.;
        self.camera.1 += screen_height() * (old_screen_zoom - new_screen_zoom) / 2.;

        self.zoom += amount;
        // let self.zoom = self.zoom.round();
    }

    fn key_down_event(&mut self, ch: char) {
        if ('1'..'9').contains(&ch) {
            let toolbar_count: usize = ch as usize - '1' as usize;
            if toolbar_count < self.toolbar_items.len() {
                let new_build_mode = self.toolbar_items[toolbar_count].build_mode;
                if self.build_mode != new_build_mode {
                    self.build_mode = new_build_mode;
                } else {
                    self.build_mode = BuildMode::None;
                }
            }
            return;
        }
        match ch {
            'q' => self.request_quit = true,
            ' ' => self.paused = !self.paused,

            '-' => self.zoom *= PLUS_MINUS_SENSITVITY,
            '=' => self.zoom /= PLUS_MINUS_SENSITVITY,

            _ => {} // }
        }
    }

    fn mouse_button_down_event(&mut self, mouse_pos: Position, map: &mut Map) {
        println!("Mouse pressed: pos: {mouse_pos:?}");
        match self.build_mode {
            BuildMode::Clear => {
                map.grid.get_tile_mut(&mouse_pos).clear();
            }
            BuildMode::Yield => {
                if let Tile::Road(road) = map.grid.get_tile_mut(&mouse_pos) {
                    road.should_yield = !road.should_yield;
                }
            }
            BuildMode::Bridge => {
                if let Some(start_pos) = self.bridge_start_pos {
                    map.grid.build_bridge(start_pos, mouse_pos);
                    self.bridge_start_pos = None;
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
    }

    fn mouse_motion_event(&mut self, pos: Position, map: &mut Map) {
        if is_mouse_button_down(MouseButton::Left) {
            match self.build_mode {
                BuildMode::AddRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        let dir = last_mouse_pos.direction_to(pos);
                        map.grid.get_tile_mut(&last_mouse_pos)
                            .edit_road(|road| road.connect(dir));
                    }
                }
                BuildMode::RemoveRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        let dir = last_mouse_pos.direction_to(pos);
                        map.grid.get_tile_mut(&last_mouse_pos)
                            .edit_road(|road| road.disconnect(dir));
                    }
                }
                BuildMode::TwoLaneRoad => {
                    if let Some(last_mouse_pos) = self.last_mouse_pos {
                        let dir = last_mouse_pos.direction_to(pos);
                        map.grid.build_two_way_road(last_mouse_pos, dir);
                    }
                }
                BuildMode::Clear => {
                    map.grid.get_tile_mut(&pos).clear();
                }
                _ => {}
            }
        }
        println!("Mouse motion, x: {}, y: {}", pos.x, pos.y);
    }
}
