use macroquad::color::{Color, WHITE};
use serde::{Deserialize, Serialize};

// pub const VIRTUAL_WIDTH: f32 = 1280.0;
// pub const VIRTUAL_HEIGHT: f32 = 720.0;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub const GAME_SPEED_NORMAL: f64 = 1./ 15.;
pub const GAME_SPEED_FAST: f64 = 1./ 60.;


#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SpawnerColors {
    None,
    Blue,
    Red,
    Green,
    Yellow,
}

impl SpawnerColors {
    pub fn color(&self) -> Color {
        match self {
            SpawnerColors::None => WHITE,
            // There is a bug with colors
            // some weird Color Luminance/Value rounding on macos
            SpawnerColors::Blue => Color::from_hex(0x8fdcea),
            SpawnerColors::Red => Color::from_hex(0xff3e42),
            SpawnerColors::Green => Color::from_hex(0x1fcf7f),
            SpawnerColors::Yellow => Color::from_hex(0xffc657),
        }
    }
    
    pub(crate) fn from_number(station: u64) -> SpawnerColors {
        match station {
            1 => SpawnerColors::Blue,
            2 => SpawnerColors::Red,
            3 => SpawnerColors::Yellow,
            4 => SpawnerColors::Green,
            _ => SpawnerColors::None,
        }
    }
}

