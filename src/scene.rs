
#[derive(Clone, Debug)]
pub enum GameOptions {
    Level(usize),
    New,
    Continue,
}


pub enum EScene {
    Gameplay(Box<Map>),
    MainMenu,
    LevelSelect,
}

use crate::{context::Context, map::Map};

// pub mod credits;
pub mod gameplay;
pub mod main_menu;
pub mod level_select;
// pub mod settings;

pub trait Scene {
    fn update(&mut self, ctx: &mut Context);
    fn draw(&mut self, ctx: &mut Context);
}