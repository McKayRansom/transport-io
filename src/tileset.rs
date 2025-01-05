use macroquad::{
    color::Color,
    math::{vec2, Rect},
    shapes::draw_rectangle,
    text::{draw_text_ex, measure_text, TextParams},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, FilterMode, Texture2D}, window::{screen_height, screen_width},
};

use crate::{grid::GRID_CELL_SIZE, map::GRID_CENTER};

const TILE_SIZE: u32 = 16;

#[derive(Clone, Copy)]
pub struct Sprite {
    pub row: u8,
    pub col: u8,
    pub size: (i8, i8),
}

impl Sprite {
    pub const fn new(row: u8, col: u8) -> Self {
        Sprite {
            row,
            col,
            size: (1, 1),
        }
    }

    pub const fn new_size(row: u8, col: u8, size: (i8, i8)) -> Self {
        Sprite { row, col, size }
    }
}

pub struct Tileset {
    pub texture: Texture2D,
    pub zoom: f32,
    pub camera: (f32, f32),
}

// TODO: Rename to TextureAtlas
impl Tileset {
    pub async fn new() -> Self {

        let texture = load_texture("resources/tileset.png").await.unwrap();
        texture.set_filter(FilterMode::Nearest);

        Tileset {
            texture,
            zoom: 1.,
            camera: (
                GRID_CENTER.0 as f32 * GRID_CELL_SIZE.0 - screen_width() / 2.,
                GRID_CENTER.1 as f32 * GRID_CELL_SIZE.1 - screen_height() / 2.,
            ),
        }
    }

    pub fn sprite_rect(&self, sprite: Sprite) -> Rect {
        Rect {
            // Adding the 0.1 margin helps avoid slight gaps between tiles
            // I'm not totally sure why, it seems to be a floating point error?
            // See: https://github.com/not-fl3/macroquad/blob/master/tiled/src/lib.rs#L80
            x: (sprite.col as u32 * TILE_SIZE) as f32 + 0.1,
            y: (sprite.row as u32 * TILE_SIZE) as f32 + 0.1,
            w: (TILE_SIZE * sprite.size.0 as u32) as f32 - 0.2,
            h: (TILE_SIZE * sprite.size.1 as u32) as f32 - 0.2,
        }
    }

    pub fn draw_tile(&self, sprite: Sprite, color: Color, dest: &Rect, rotation: f32) {
        let dest_size = vec2(
            dest.w * sprite.size.0 as f32 * self.zoom,
            dest.h * sprite.size.1 as f32 * self.zoom,
        );
        let spr_rect = self.sprite_rect(sprite);

        draw_texture_ex(
            &self.texture,
            (dest.x - self.camera.0) * self.zoom,
            (dest.y - self.camera.1) * self.zoom,
            color,
            DrawTextureParams {
                dest_size: Some(dest_size),
                source: Some(spr_rect),
                rotation,
                ..Default::default()
            },
        );
    }

    pub fn draw_rect(&self, rect: &Rect, color: Color) {
        draw_rectangle(
            (rect.x - self.camera.0) * self.zoom,
            (rect.y - self.camera.1) * self.zoom,
            rect.w * self.zoom,
            rect.h * self.zoom,
            color,
        );
    }

    /// Draws text centered
    pub fn draw_text(&self, text: &str, text_size: f32, color: Color, rect: &Rect) {
        let font_size = (text_size * self.zoom) as u16;
        let text_measured = measure_text(text, None, font_size, 1.0);
        draw_text_ex(
            text,
            (rect.x - self.camera.0) * self.zoom - text_measured.width / 2.,
            (rect.y - self.camera.1) * self.zoom + text_measured.height / 2.,
            TextParams {
                font_size,
                font_scale: 1.0,
                color,
                ..Default::default()
            },
        );
    }
}
