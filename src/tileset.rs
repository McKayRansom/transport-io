use macroquad::{color::Color, math::{vec2, Rect}, texture::{draw_texture_ex, DrawTextureParams, Texture2D}};

use crate::grid::{Direction, Rectangle};

const TILE_SIZE: u32 = 16;


pub struct Tileset {
    texture: Texture2D,
    columns: u32,
}

impl Tileset {
    pub fn new(texture: Texture2D, columns: u32) -> Self {
        
        Tileset { texture, columns }
    }

    fn sprite_rect(&self, sprite: u32) -> Rectangle {
        Rectangle {
            x: ((sprite % self.columns) * TILE_SIZE) as f32,
            y: ((sprite / self.columns) * TILE_SIZE) as f32,
            w: TILE_SIZE as f32,
            h: TILE_SIZE as f32,
        }
    }

    pub fn draw_tile(&self, sprite: u32, color: Color, dest: &Rectangle, rotation: f32)
    {

        let spr_rect = self.sprite_rect(sprite);

        draw_texture_ex(
            &self.texture,
            dest.x,
            dest.y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(Rect::new(
                    spr_rect.x, // spr_rect.x + 1.1 TODO: WHY was it like this before?
                    spr_rect.y,
                    spr_rect.w,
                    spr_rect.h,
                )),
                rotation: rotation,
                ..Default::default()
            },
        );
    }

}
