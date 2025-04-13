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
       
        self.game_view.update(ctx, map);
        self.game_view.draw(map, ctx);
    }
}
