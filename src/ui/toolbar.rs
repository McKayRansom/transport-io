use macroquad::{
    color::{Color, WHITE}, input::{is_mouse_button_down, mouse_position}, math::{vec2, Rect}, shapes::draw_rectangle, texture::{draw_texture_ex, DrawTextureParams}, window::{screen_height, screen_width}
};

use crate::tileset::{Sprite, Tileset};

#[derive(Clone, Copy)]
pub struct ToolbarItem<V> {
    value: V,
    _tooltip: &'static str,
    shortcut: char,
    sprite: Sprite,
}

impl<V> ToolbarItem<V> {
    pub fn new(value: V, _tooltip: &'static str, shortcut: char, sprite: Sprite) -> Self {
        Self {
            value,
            _tooltip,
            shortcut,
            sprite,
        }
    }
}

pub struct Toolbar<V> {
    selected: Option<usize>,
    items: Vec<ToolbarItem<V>>,
    rect: Rect,
}

impl<V> Toolbar<V> {
    pub fn new(items: Vec<ToolbarItem<V>>) -> Self {
        Self {
            rect: Rect::new(0., 0., 0., 0.),
            selected: None,
            items,
        }
    }

    pub fn get_selected(&self) -> Option<&V> {
        let selected = self.selected?;
        Some(&self.items[selected].value)
    }

    pub fn draw(&mut self, tileset: &Tileset) {
        let toolbar_item_count: f32 = 5.;
        let toolbar_item_width: f32 = 64.;
        let toolbar_item_height: f32 = 64.;
        let toolbar_item_pad: f32 = 10.;

        let toolbar_height: f32 = toolbar_item_height + toolbar_item_pad;
        let toolbar_width = (toolbar_item_width + toolbar_item_pad) * toolbar_item_count;

        // let mut build_mode = self.build_mode;
        self.rect.x = screen_width() / 2.0 - (toolbar_width / 2.);
        self.rect.y = screen_height() - toolbar_height;

        self.rect.w = toolbar_width;
        self.rect.h = toolbar_height;

        let window_color = Color::from_hex(0x585858);

        draw_rectangle(
            self.rect.x,
            self.rect.y,
            toolbar_width,
            toolbar_height,
            window_color,
        );

        let mut item_rect = Rect::new(
            self.rect.x + toolbar_item_pad / 2.,
            self.rect.y + toolbar_item_pad / 2.,
            toolbar_item_width,
            toolbar_item_height,
        );
        for (i, toolbar_item) in self.items.iter().enumerate() {
            let spr_rect = tileset.sprite_rect(toolbar_item.sprite);

            draw_texture_ex(
                &tileset.texture,
                item_rect.x,
                item_rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(item_rect.w, item_rect.h)),
                    source: Some(spr_rect),
                    ..Default::default()
                },
            );

            if self.selected == Some(i) {
                draw_rectangle(
                    item_rect.x,
                    item_rect.y,
                    item_rect.w,
                    item_rect.h,
                    Color::new(0.0, 0.0, 0.0, 0.1),
                );
            }

            if item_rect.contains(mouse_position().into()) {
                draw_rectangle(
                    item_rect.x,
                    item_rect.y,
                    item_rect.w,
                    item_rect.h,
                    Color::new(0.0, 0.0, 0.0, 0.1),
                );

                if is_mouse_button_down(macroquad::input::MouseButton::Left) {
                    self.selected = Some(i);
                }
            }

            item_rect.x += toolbar_item_width + toolbar_item_pad;
        }
    }

    pub fn key_down(&mut self, key: char) {
        for (i, item) in self.items.iter().enumerate() {
            if key == item.shortcut {
                if let Some(selected) = self.selected {
                    if selected == i {
                        self.selected = None;
                    } else {
                        self.selected = Some(i);
                    }
                } else {
                    self.selected = Some(i);
                }
                return;
            }
        }
    }

    pub fn is_mouse_over(&self, mouse_pos: (f32, f32)) -> bool {
        self.rect.contains(mouse_pos.into())
    }
}
