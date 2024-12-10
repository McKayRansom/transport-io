use macroquad::{
    color::Color,
    math::{vec2, Rect},
    shapes::draw_rectangle,
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

const TILE_SIZE: u32 = 16;

pub struct Tileset {
    texture: Texture2D,
    columns: u32,
    pub zoom: f32,
    pub camera: (f32, f32),
}

impl Tileset {
    pub fn new(texture: Texture2D, columns: u32) -> Self {
        let zoom = 0.0;
        let camera = (0.0, 0.0);
        Tileset {
            texture,
            columns,
            zoom,
            camera,
        }
    }

    fn sprite_rect(&self, sprite: u32) -> Rect {
        Rect {
            // spr_rect.x + 1.1 TODO: WHY was it like this before? A: Maybe due to weird drawing in webasm?
            x: ((sprite % self.columns) * TILE_SIZE) as f32,
            y: ((sprite / self.columns) * TILE_SIZE) as f32,
            w: TILE_SIZE as f32,
            h: TILE_SIZE as f32,
        }
    }

    pub fn draw_tile(&self, sprite: u32, color: Color, dest: &Rect, rotation: f32) {
        let spr_rect = self.sprite_rect(sprite);

        draw_texture_ex(
            &self.texture,
            (dest.x - self.camera.0) * self.zoom,
            (dest.y - self.camera.1) * self.zoom,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w * self.zoom, dest.h * self.zoom)),
                source: Some(spr_rect),
                rotation: rotation,
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
