use macroquad::{
    color::Color,
    math::{vec2, Rect},
    shapes::draw_rectangle,
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

const TILE_SIZE: u32 = 16;

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
    texture: Texture2D,
    pub zoom: f32,
    pub camera: (f32, f32),
}

impl Tileset {
    pub fn new(texture: Texture2D) -> Self {
        let zoom = 0.0;
        let camera = (0.0, 0.0);
        Tileset {
            texture,
            zoom,
            camera,
        }
    }

    fn sprite_rect(&self, sprite: Sprite) -> Rect {
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

    pub fn draw_tile_flip(&self, sprite: Sprite, color: Color, dest: &Rect, flip: bool) {
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
                flip_x: flip,
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
}
