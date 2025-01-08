use macroquad::{
    math::vec2,
    text::measure_text,
    ui::{hash, root_ui, widgets},
    window::{screen_height, screen_width},
};

use crate::ui::skin::{MENU_FONT_SIZE, MENU_MARGIN};

#[derive(Clone)]
pub struct MenuItem<V> {
    value: V,
    label: String,
}

impl<V> MenuItem<V> {
    pub fn new(value: V, label: String) -> Self {
        Self { value, label }
    }
}

pub struct Menu<V> {
    pub items: Vec<MenuItem<V>>,
}

impl<V> Menu<V> {
    pub fn new(items: Vec<MenuItem<V>>) -> Self {
        Self { items }
    }

    pub fn draw(&self) -> Option<&V> {
        let menu_width =
            measure_text(&self.items[0].label, None, MENU_FONT_SIZE, 1.).width + MENU_MARGIN * 6.;
        let menu_height = 300.;

        let mut selected = None;

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
            for item in &self.items {
                if ui.button(None, item.label.as_str()) {
                    selected = Some(&item.value);
                }
            }
        });

        selected
    }
}
