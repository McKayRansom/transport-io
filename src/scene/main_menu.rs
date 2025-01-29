// use super::credits::Credits;
// use super::settings::Settings;
use super::{EScene, Scene};
// use crate::audio::play_sfx;
use crate::consts::*;
use crate::context::Context;
use crate::map::draw::draw_map;
use crate::map::position::GRID_CELL_SIZE;
use crate::map::{Map, DEFAULT_CITY_ID};
use crate::ui::{
    menu::{Menu, MenuItem},
    popup::Popup,
};
// use crate::ui::skin::{MENU_FONT_SIZE, MENU_MARGIN};
// use crate::input::{action_pressed, Action};
// use crate::text::{self, draw_text};
use macroquad::color::{BLACK, WHITE};
// use macroquad::math::vec2;
use macroquad::text::{draw_text, draw_text_ex, measure_text};
use macroquad::time::get_time;
use macroquad::ui::hash;
// use macroquad::ui::{hash, root_ui, widgets};
use macroquad::window::{screen_height, screen_width};

const MAIN_MENU_MAP: &str = "
__________________
__________________
__________________
__________________
________11________
________11________
____lr<<lr<<lr____
____LR>>LR>>LR____
____.^__.^__.^____
____.^__.^__.^____
____.^__.^__.^____
____.^__.^__.^____
22<<lr<<lr<<lr<<33
22>>LR>>LR>>LR>>33
____.^__.^__.^____
____.^__.^__.^____
____.^__.^__.^____
____.^__.^__.^____
____lr<<lr<<lr____
____LR>>LR>>LR____
________.^________
________.^________
________44________
________44________
";

#[derive(Clone)]
enum MenuOption {
    Continue,
    Start,
    Levels,
    Freeplay,
    // Settings,
    // Credits,
    #[cfg(not(target_family = "wasm"))]
    Quit,
}

pub struct MainMenu {
    menu: Menu<MenuOption>,
    // settings_subscene: Settings,
    // credits_subscene: Credits,
    popup: Option<Popup>,

    last_update: f64,
    map: Map,
}

impl MainMenu {
    pub async fn new(_ctx: &mut Context) -> Self {
        let mut main_menu = Self {
            menu: Menu::new(vec![
                if Map::save_exists() {
                    MenuItem::new(MenuOption::Continue, "Continue".to_string())
                } else {
                    MenuItem::new(MenuOption::Start, "Start".to_string())
                },
                MenuItem::new(MenuOption::Levels, "Levels".to_string()),
                MenuItem::new(MenuOption::Freeplay, "Freeplay".to_string()),
                #[cfg(not(target_family = "wasm"))]
                MenuItem::new(MenuOption::Quit, "Quit".to_string()),
            ]),
            popup: None,
            // settings_subscene: Settings::new(ctx, false),
            // credits_subscene: Credits::new(ctx),
            map: Map::new_from_string(MAIN_MENU_MAP),
            last_update: 0.,
        };

        main_menu.map.get_city_mut(DEFAULT_CITY_ID).unwrap().name = "Alpha 0.1X - Roads".into();

        main_menu
    }

    fn menu_option_selected(&mut self, menu_option: MenuOption, ctx: &mut Context) {
        match menu_option {
            MenuOption::Continue => match super::GameOptions::Continue.create() {
                Ok(map) => ctx.switch_scene_to = Some(EScene::Gameplay(Box::new(map))),
                Err(_) => self.popup = Some(Popup::new("Error loading save".into())),
            },
            MenuOption::Start => {
                ctx.switch_scene_to = Some(EScene::Gameplay(Box::new(
                    super::GameOptions::Level(0)
                        .create()
                        .expect("Error loading level"),
                )))
            }
            MenuOption::Levels => {
                ctx.switch_scene_to = Some(EScene::LevelSelect);
            }
            MenuOption::Freeplay => {
                ctx.switch_scene_to = Some(EScene::Gameplay(Box::new(
                    super::GameOptions::New
                        .create()
                        .expect("Error generating map"),
                )))
            }
            // MenuOption::Settings => {
            //     self.settings_subscene.active = true;
            // }
            // MenuOption::Credits => {
            //     self.credits_subscene.active = true;
            // }
            #[cfg(not(target_family = "wasm"))]
            MenuOption::Quit => {
                ctx.request_quit = true;
            }
        }
    }
}

impl Scene for MainMenu {
    fn update(&mut self, _ctx: &mut Context) {
        // if self.settings_subscene.active {
        //     self.settings_subscene.update(ctx);
        //     return;
        // }

        // if self.credits_subscene.active {
        //     self.credits_subscene.update(ctx);
        //     return;
        // }
        self.map.metadata.grow_cities = false;
        let speed = 1. / 4.;

        if get_time() - self.last_update > speed {
            self.last_update = get_time();
            self.map.update();
        }
    }
    fn draw(&mut self, ctx: &mut Context) {
        // if self.settings_subscene.active {
        //     self.settings_subscene.draw(ctx);
        //     return;
        // }

        // if self.credits_subscene.active {
        //     self.credits_subscene.draw(ctx);
        //     return;
        // }

        ctx.tileset
            .reset_camera((GRID_CELL_SIZE.0 * 18., GRID_CELL_SIZE.1 * 20.));
        ctx.tileset.change_zoom(1.2);

        draw_map(&self.map, &ctx.tileset);

        let menu_height = 200.;

        let font_size: u16 = 120;

        let measure = measure_text("Transport IO", Some(&ctx.font), font_size, 1.);

        let x = -measure.width / 2. + screen_width() / 2.;
        let y = -menu_height + screen_height() / 2.;

        let shadow_y = y + 5.;
        let shadow_x = x + 5.;

        draw_text_ex(
            "Transport IO",
            shadow_x,
            shadow_y,
            macroquad::text::TextParams {
                font: Some(&ctx.font),
                font_size,
                color: BLACK,
                ..Default::default()
            },
        );

        draw_text_ex(
            "Transport IO",
            x,
            y,
            macroquad::text::TextParams {
                font: Some(&ctx.font),
                font_size,
                color: WHITE,
                ..Default::default()
            },
        );

        // draw_text("Transport IO", x, y, font_size, WHITE);

        if let Some(selected) = self.menu.draw(hash!()).cloned() {
            self.menu_option_selected(selected, ctx);
        }

        draw_text(
            // ctx,
            format!("v{}", VERSION).as_str(),
            40.,
            screen_height() - 40.,
            // text::Size::Small,
            20.,
            WHITE,
        );

        if let Some(popup) = &self.popup {
            match popup.draw() {
                None => {}
                Some(crate::ui::popup::PopupResult::Ok) => self.popup = None,
                Some(crate::ui::popup::PopupResult::Cancel) => self.popup = None,
            }
        }
    }
}
