use macroquad::{input::{get_char_pressed, mouse_position}, math::vec2, ui::root_ui};
use view_game::GameView;

use crate::{
    context::Context,
    map::{
        Map, Unlocked,
    },
};

mod build_history;
pub mod menu;
pub mod popup;
pub mod skin;
mod toolbar;
pub mod view_game;
mod view_build;
mod view_route;


pub struct UiState {
    game_view: GameView,
}

impl UiState {
    pub fn new(unlocked: Unlocked) -> Self {
        Self {
            game_view: GameView::new(unlocked)
        }
    }

    pub fn draw(&mut self, map: &mut Map, ctx: &mut Context) {
        let mouse_pos = mouse_position();
       
        if root_ui().is_mouse_over(vec2(mouse_pos.0, mouse_pos.1)) {
            ctx.mouse_pos = None;
        } else {
            ctx.mouse_pos = Some(mouse_pos.into())
        }

        ctx.key_pressed = get_char_pressed();

        self.game_view.update(ctx, map);
        self.game_view.draw(map, ctx);
    }
}
