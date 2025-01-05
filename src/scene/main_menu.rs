// use super::credits::Credits;
// use super::settings::Settings;
use super::{EScene, Scene};
// use crate::audio::play_sfx;
use crate::consts::*;
use crate::context::Context;
use crate::ui::menu::{Menu, MenuItem};
use crate::ui::skin::{MENU_FONT_SIZE, MENU_MARGIN};
// use crate::input::{action_pressed, Action};
// use crate::text::{self, draw_text};
use macroquad::color::{BLACK, RED, WHITE};
use macroquad::math::vec2;
use macroquad::text::{draw_text, measure_text};
use macroquad::ui::{hash, root_ui, widgets};
use macroquad::window::{screen_height, screen_width};

enum MenuOption {
    Play,
    // Settings,
    // Credits,
    #[cfg(not(target_family = "wasm"))]
    Quit,
}

pub struct MainMenu {
    menu: Menu<MenuOption>,
    // settings_subscene: Settings,
    // credits_subscene: Credits,
}

impl MainMenu {
    pub async fn new(_ctx: &mut Context) -> Self {
        Self {
            menu: Menu::new(vec![
                MenuItem::new(MenuOption::Play, "Play"),
                #[cfg(not(target_family = "wasm"))]
                MenuItem::new(MenuOption::Quit, "Quit"),
            ])
            // settings_subscene: Settings::new(ctx, false),
            // credits_subscene: Credits::new(ctx),
        }
    }

    fn menu_option_selected(&self, menu_option: &MenuOption, ctx: &mut Context) {
        match menu_option {
            MenuOption::Play => {
                ctx.switch_scene_to = Some(EScene::Gameplay);
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

        let x =-300. + screen_width() / 2.;
        let y = -menu_height + screen_height() / 2.;
        let font_size = 120.0;
    
        let shadow_y = y + 5.;
        let shadow_x = x + 5.;
    
        draw_text(
            "Transport IO",
            shadow_x, shadow_y, font_size,
            BLACK,
        );
    
        draw_text(
            "Transport IO",
            x, y, font_size,
            WHITE,
        );
    
        if let Some(selected) = self.menu.draw() {
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
    }
}
