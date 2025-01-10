use macroquad::{
    math::vec2,
    text::measure_text,
    ui::{hash, root_ui, widgets},
    window::{screen_height, screen_width},
};

use crate::ui::skin::{MENU_FONT_SIZE, MENU_MARGIN};

pub struct Popup {
    message: String,
}

pub enum PopupResult {
    Ok,
    Cancel,
}

impl Popup {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn draw(&self) -> Option<PopupResult> {
        let menu_width = 
            measure_text(&self.message, None, MENU_FONT_SIZE, 1.).width + MENU_MARGIN * 4.;
        let menu_height = 300.;

        let mut selected: Option<PopupResult> = None;

        widgets::Window::new(
            hash!(),
            vec2(
                screen_width() / 2.0 - (menu_width / 2.),
                screen_height() / 2.0 - (menu_height / 2.),
            ),
            vec2(menu_width, menu_height),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), |ui| {
            ui.label(None, &self.message);
            // ui.mu
            // for item in &self.items {
                if ui.button(None, "Ok") {
                    selected = Some(PopupResult::Ok);
                }
                if ui.button(None, "Cancel") {
                    selected = Some(PopupResult::Cancel);
                }
            // }
        });

        selected
    }
}
