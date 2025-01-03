use macroquad::{color::WHITE, prelude::rand};
use serde::{Deserialize, Serialize};

use crate::{
    grid::{Id, Position},
    tileset::Tileset,
};

const CITY_GROW_TICKS: u32 = 16 * 10;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct City {
    pos: Position,
    name: String,
    // pub rating: f32,
    pub grow_ticks: u32,
    id: Id,
    pub houses: Vec<Id>,
    // vehicle_on_the_way: Option<Id>,
}

impl City {
    pub fn new(id: Id, pos: Position, name: String) -> Self {
        City {
            pos: pos,
            name: name,
            grow_ticks: 0,
            id: id,
            houses: Vec::new(),
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        tileset.draw_text(self.name.as_str(), 32., WHITE, &self.pos.into());
    }

    pub fn update(&mut self) -> bool {
        self.grow_ticks += 1;
        // TODO: Grow conditions??
        if self.grow_ticks > CITY_GROW_TICKS {
            self.grow_ticks = 0;
            true
        } else {
            false
        }
    }

    pub fn random_house(&self) -> Id {
        let index = rand::gen_range(0, self.houses.len());
        self.houses[index]
    }
}
