use macroquad::{
    color::{Color, WHITE},
    input::{is_mouse_button_down, mouse_position},
    math::{vec2, Rect},
    shapes::draw_rectangle,
    texture::{draw_texture_ex, DrawTextureParams},
};

use crate::{context::Context, tileset::Sprite};

#[derive(Clone, Copy)]
pub struct ToolbarItem<V> {
    value: V,
    _tooltip: &'static str,
    shortcut: char,
    pub sprite: Sprite,
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

const TOOLBAR_ITEM_WIDTH: f32 = 64.;
const TOOLBAR_ITEM_HEIGHT: f32 = 64.;
const TOOLBAR_ITEM_PAD: f32 = 10.;

pub const TOOLBAR_SPACE: f32 = TOOLBAR_ITEM_HEIGHT + TOOLBAR_ITEM_PAD;
// pub const TOOLBAR_WIDTH: f32 = (TOOLBAR_ITEM_WIDTH + TOOLBAR_ITEM_PAD) * TOOLBAR_ITEM_COUNT;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolbarType {
    Horizontal,
    Veritcal,
}

pub struct Toolbar<V> {
    kind: ToolbarType,
    selected: Option<usize>,
    mouse_down: Option<usize>,
    pub items: Vec<ToolbarItem<V>>,
    rect: Rect,
}

impl<V> Toolbar<V> {
    pub fn new(kind: ToolbarType, items: Vec<ToolbarItem<V>>) -> Self {
        Self {
            kind,
            selected: None,
            mouse_down: None,
            items,
            rect: Rect::new(0., 0., 0., 0.),
        }
    }

    pub fn get_selected(&self) -> Option<&V> {
        let selected = self.selected?;
        Some(&self.items[selected].value)
    }

    pub fn draw(&mut self, ctx: &Context, x: f32, y: f32) {

        if self.kind == ToolbarType::Veritcal {
            self.rect.w = TOOLBAR_SPACE;
            self.rect.h = self.items.len() as f32 * TOOLBAR_SPACE;
            self.rect.x = x;
            self.rect.y = y - self.rect.h / 2.;
        } else {
            self.rect.w = self.items.len() as f32 * TOOLBAR_SPACE;
            self.rect.h = TOOLBAR_SPACE;
            self.rect.x = x - self.rect.w / 2.;
            self.rect.y = y;
        }

        let window_color = Color::from_hex(0x585858);
        let mouse_down = is_mouse_button_down(macroquad::input::MouseButton::Left);

        if !mouse_down {
            self.mouse_down = None;
        }

        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            window_color,
        );

        let mut item_rect = Rect::new(
            self.rect.x + TOOLBAR_ITEM_PAD / 2.,
            self.rect.y + TOOLBAR_ITEM_PAD / 2.,
            TOOLBAR_ITEM_WIDTH,
            TOOLBAR_ITEM_HEIGHT,
        );
        for (i, toolbar_item) in self.items.iter().enumerate() {
            let spr_rect = ctx.tileset.sprite_rect(toolbar_item.sprite);

            draw_texture_ex(
                &ctx.tileset.texture,
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

                if mouse_down {
                    if self.mouse_down == None {
                        if self.selected == None {
                            self.selected = Some(i);
                        } else if self.selected == Some(i) {
                            self.selected = None;
                        } else {
                            self.selected = Some(i);
                        }
                        self.mouse_down = Some(i);
                    }
                }
            }

            if self.kind == ToolbarType::Horizontal {
                item_rect.x += TOOLBAR_ITEM_WIDTH + TOOLBAR_ITEM_PAD;
            } else {
                item_rect.y += TOOLBAR_ITEM_WIDTH + TOOLBAR_ITEM_PAD;
            }
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
