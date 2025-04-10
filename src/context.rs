use macroquad::math::Vec2;
use macroquad::text::Font;
use macroquad::time::get_time;

// use crate::assets_path::determine_asset_path;
// use crate::audio;
// use crate::consts::*;
// use crate::font;
// use crate::save::Save;
use crate::scene::EScene;
// use crate::settings::Settings;
use crate::tileset::Tileset;
use crate::ui::skin;
// use macroquad::math::Rect;
// use macroquad::miniquad::FilterMode;
// use macroquad::texture::render_target;
// use macroquad::camera::Camera2D;

pub enum GameSpeed {
    Paused,
    Normal,
    FastForward,
}

/// game-wide data and resources
pub struct Context {
    pub request_quit: bool,
    pub key_pressed: Option<char>,
    pub mouse_pos: Option<Vec2>,
    pub _time: f64,
    pub game_speed: GameSpeed,
    // pub gamepads: Gamepads,
    // pub textures: texture::TextureAtlas,
    pub tileset: Tileset,
    // pub fonts: font::FontAtlas,
    pub font: Font,
    // pub audio: audio::AudioAtlas,
    // pub render_target: RenderTarget,
    // pub render_target_cam: Camera2D,
    pub switch_scene_to: Option<EScene>,
    // pub settings: Settings,
    // pub save: Save,
}

impl Context {
    pub async fn default() -> Self {

        Self {
            // gamepads: Gamepads::new(),
            game_speed: GameSpeed::Paused,
            key_pressed: None,
            mouse_pos: None,
            _time: get_time(),
            request_quit: false,
            tileset: Tileset::new().await,
            font: skin::init().await,
            // audio: audio::AudioAtlas::new().await,
            // fonts: font::FontAtlas::new().await,
            // render_target,
            // render_target_cam,
            switch_scene_to: None,
            // settings: Settings::load(),
            // save: Save::load(),
        }
    }
}
