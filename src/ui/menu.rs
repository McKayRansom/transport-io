use macroquad::{
    math::{vec2, Vec2},
    text::{measure_text, TextDimensions},
    ui::{
        root_ui,
        widgets::{self, Button},
        Id,
    },
    window::{screen_height, screen_width},
};

use crate::ui::skin::{MENU_FONT_SIZE, MENU_MARGIN};

use super::skin::{BUTTON_MARGIN, MENU_OUTER_MARGIN};

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

    pub fn draw(&self, id: Id) -> Option<&V> {
        let measure = self
            .items
            .iter()
            .map(|item| measure_text(item.label.as_str(), None, MENU_FONT_SIZE, 1.))
            .fold(TextDimensions::default(), |accum, elem| TextDimensions {
                width: accum.width.max(elem.width),
                height: accum.height.max(elem.height),
                offset_y: accum.offset_y.max(elem.offset_y),
            });

        let button_size = Vec2::new(
            measure.width + BUTTON_MARGIN.0 * 2.,
            measure.height + BUTTON_MARGIN.1 * 2.,
        );

        let menu_width = MENU_MARGIN * 4. + button_size.x + MENU_OUTER_MARGIN * 2.;
        let menu_height =
            (MENU_MARGIN + button_size.y) * self.items.len() as f32 + MENU_MARGIN * 3. + MENU_OUTER_MARGIN * 2.;

        let mut selected = None;

        widgets::Window::new(
            id,
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
                if Button::new(item.label.as_str())
                    .position(None)
                    .size(button_size)
                    .ui(ui)
                {
                    selected = Some(&item.value);
                }
            }
        });

        selected
    }
}
