use macroquad::color::{Color, WHITE};
use serde::{Deserialize, Serialize};

// pub const VIRTUAL_WIDTH: f32 = 1280.0;
// pub const VIRTUAL_HEIGHT: f32 = 720.0;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");


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
            SpawnerColors::Blue => Color::from_hex(0xa0dae8),
            SpawnerColors::Red => Color::from_hex(0xf9524c),
            SpawnerColors::Green => Color::from_hex(0x62cc86),
            SpawnerColors::Yellow => Color::from_hex(0xf8c86a),
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

