// use super::credits::Credits;
// use super::settings::Settings;
use super::{EScene, Scene};
// use crate::audio::play_sfx;
use crate::consts::*;
use crate::context::Context;
use crate::ui::{
    menu::{Menu, MenuItem},
    popup::Popup,
};
// use crate::ui::skin::{MENU_FONT_SIZE, MENU_MARGIN};
// use crate::input::{action_pressed, Action};
// use crate::text::{self, draw_text};
use macroquad::color::{BLACK, WHITE};
// use macroquad::math::vec2;
use macroquad::text::draw_text;
// use macroquad::ui::{hash, root_ui, widgets};
use macroquad::window::{screen_height, screen_width};

#[derive(Clone)]
enum MenuOption {
    Continue,
    Scenarios,
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
}

impl MainMenu {
    pub async fn new(_ctx: &mut Context) -> Self {
        Self {
            menu: Menu::new(vec![
                MenuItem::new(MenuOption::Continue, "Continue".to_string()),
                MenuItem::new(MenuOption::Scenarios, "Scenarios".to_string()),
                MenuItem::new(MenuOption::Freeplay, "Freeplay".to_string()),
                #[cfg(not(target_family = "wasm"))]
                MenuItem::new(MenuOption::Quit, "Quit".to_string()),
            ]),
            popup: None,
            // settings_subscene: Settings::new(ctx, false),
            // credits_subscene: Credits::new(ctx),
        }
    }

    fn menu_option_selected(&mut self, menu_option: MenuOption, ctx: &mut Context) {
        match menu_option {
            MenuOption::Continue => match super::GameOptions::Continue.create() {
                Ok(map) => ctx.switch_scene_to = Some(EScene::Gameplay(Box::new(map))),
                Err(_) => self.popup = Some(Popup::new("Error loading save".into())),
            },
            MenuOption::Scenarios => {
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

        let menu_height = 300.;

        let x = -300. + screen_width() / 2.;
        let y = -menu_height + screen_height() / 2.;
        let font_size = 120.0;

        let shadow_y = y + 5.;
        let shadow_x = x + 5.;

        draw_text("Transport IO", shadow_x, shadow_y, font_size, BLACK);

        draw_text("Transport IO", x, y, font_size, WHITE);

        if let Some(selected) = self.menu.draw().cloned() {
            self.menu_option_selected(selected, ctx);
        }

        draw_text(
            // ctx,
            format!("v{}", VERSION).as_str(),
            40.,
            VIRTUAL_HEIGHT - 40.,
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
