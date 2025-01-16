use crate::{
    context::Context,
    map::levels::LEVEL_COUNT,
    ui::menu::{Menu, MenuItem},
};

use super::{EScene, GameOptions, Scene};

pub struct LevelSelect {
    menu: Menu<i32>,
}

impl LevelSelect {
    pub fn new(_ctx: &mut Context) -> Self {
        let mut menu: Menu<i32> = Menu::new(Vec::new());
        for i in 0..LEVEL_COUNT {
            menu.items
                .push(MenuItem::new(i as i32, format!("{i}").to_string()));
        }
        Self { menu }
    }

    fn level_selected(&self, selected: &i32, ctx: &mut Context) {
        ctx.switch_scene_to = Some(EScene::Gameplay(Box::new(
            GameOptions::Level(*selected as usize)
                .create()
                .expect("Error loading Level"),
        )));
    }
}

impl Scene for LevelSelect {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        if let Some(selected) = self.menu.draw() {
            self.level_selected(selected, ctx);
        }
    }
}
