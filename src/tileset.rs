use macroquad::{
    color::{colors, Color, BLACK},
    math::{vec2, Rect},
    shapes::draw_rectangle,
    text::{draw_text_ex, measure_text, TextParams},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, FilterMode, Texture2D},
    window::{screen_height, screen_width},
};

use crate::map::Direction;

const TILE_SIZE: u32 = 16;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 4.;

const SHADOW_OFFSET: f32 = 1.;

#[derive(Clone, Copy)]
pub struct Sprite {
    pub row: u8,
    pub col: u8,
    pub size: Direction,
}

impl Sprite {
    pub const fn new(row: u8, col: u8) -> Self {
        Sprite {
            row,
            col,
            size: Direction::new(1, 1, 0),
        }
    }

    pub const fn new_size(row: u8, col: u8, size: Direction) -> Self {
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
            camera: (0., 0.),
        }
    }

    pub fn reset_camera(&mut self, size: (f32, f32)) {
        self.camera = (
            -(screen_width() - size.0) / 2.,
            -(screen_height() - size.1) / 2.,
        );
        self.zoom = 1.;
        let zoom = (screen_height() / size.1).min(screen_width() / size.0);
        self.change_zoom(zoom - self.zoom);
    }

    pub fn change_zoom(&mut self, amount: f32) {
        let new_zoom = self.zoom + amount;

        if new_zoom <= MIN_ZOOM || new_zoom >= MAX_ZOOM {
            return;
        }

        let old_screen_zoom = 1. / self.zoom;
        let new_screen_zoom = 1. / new_zoom;
        self.camera.0 += screen_width() * (old_screen_zoom - new_screen_zoom) / 2.;
        self.camera.1 += screen_height() * (old_screen_zoom - new_screen_zoom) / 2.;

        self.zoom += amount;
        // let self.zoom = self.zoom.round();
    }

    pub fn sprite_rect(&self, sprite: Sprite) -> Rect {
        Rect {
            // Adding the 0.1 margin helps avoid slight gaps between tiles
            // I'm not totally sure why, it seems to be a floating point error?
            // See: https://github.com/not-fl3/macroquad/blob/master/tiled/src/lib.rs#L80
            x: (sprite.col as u32 * TILE_SIZE) as f32 + 0.1,
            y: (sprite.row as u32 * TILE_SIZE) as f32 + 0.1,
            w: (TILE_SIZE * sprite.size.x as u32) as f32 - 0.2,
            h: (TILE_SIZE * sprite.size.y as u32) as f32 - 0.2,
        }
    }

    pub fn draw_tile(&self, sprite: Sprite, color: Color, dest: &Rect, rotation: f32) {
        self.draw_tile_ex(sprite, color, dest, rotation, false);
    }

    pub fn draw_tile_flip(&self, sprite: Sprite, color: Color, dest: &Rect, rotation: f32) {
        self.draw_tile_ex(sprite, color, dest, rotation, true);
    }

    pub fn draw_tile_ex(&self, sprite: Sprite, color: Color, dest: &Rect, rotation: f32, flip: bool) {
        let dest_size = vec2(
            dest.w * sprite.size.x as f32 * self.zoom,
            dest.h * sprite.size.y as f32 * self.zoom,
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

    /// Draws text centered
    pub fn draw_text(&self, text: &str, text_size: f32, color: Color, rect: &Rect) {
        let font_size = (text_size * self.zoom) as u16;
        let text_measured = measure_text(text, None, font_size, 1.0);

        let rect: Rect = Rect::new(rect.x + rect.w / 2., rect.y + rect.h / 2., 0., 0.);

        let shadow_x =
            (rect.x + SHADOW_OFFSET - self.camera.0) * self.zoom - text_measured.width / 2.;
        let shadow_y =
            (rect.y + SHADOW_OFFSET - self.camera.1) * self.zoom + text_measured.height / 2.;
        draw_text_ex(
            text,
            shadow_x,
            shadow_y,
            TextParams {
                font_size,
                font_scale: 1.0,
                color: BLACK,
                ..Default::default()
            },
        );

        let x = (rect.x - self.camera.0) * self.zoom - text_measured.width / 2.;
        let y = (rect.y - self.camera.1) * self.zoom + text_measured.height / 2.;
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font_size,
                font_scale: 1.0,
                color,
                ..Default::default()
            },
        );
    }

    pub fn draw_icon(&self, sprite: Sprite, rect: &Rect, rotation: f32) {
        let mut rect = *rect;
        rect.w -= 16.;
        rect.h -= 16.;
        rect.x += 8.;
        rect.y += 8.;
        let mut shadow_rect = rect;
        shadow_rect.x += 1.;
        shadow_rect.y += 1.;
        // self.draw_tile(sprite, colors::BLACK, &shadow_rect, rotation);

        self.draw_tile(sprite, colors::WHITE, &rect, rotation);
    }
}
