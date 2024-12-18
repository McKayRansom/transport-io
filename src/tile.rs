
mod road;
use macroquad::{color::{Color, WHITE}, math::Rect};
pub use road::*;

use crate::{grid::{ConnectionsIterator, Id, Position}, tileset::Tileset};

const HOUSE_SPRITE: u32 = (16 * 1) + 0;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct House {
    pub vehicle_on_the_way: Option<Id>,
}

impl House {
    pub fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let color = if self.vehicle_on_the_way.is_some() {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        tileset.draw_tile(HOUSE_SPRITE, color, &rect, 0.0);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    House(House),
    Road(Road),
}

impl Tile {
    pub fn new() -> Tile {
        Tile::Empty
    }

    pub fn iter_connections(&self) -> ConnectionsIterator {
        match self {
            Tile::Road(road) => road.connections.iter(),
            Tile::House(_) => ConnectionsIterator::all_directions(),
            Tile::Empty => ConnectionsIterator::no_directions(),
        }
    }
    pub fn draw(&self, pos: Position, tileset: &Tileset) {
        let rect = Rect::from(pos);

        match self {
            Tile::Road(road) => road.draw(&rect, tileset),
            _ => {}
        }
    }

    // pub fn should_yield(&self) -> bool {
    //     match self {
    //         Tile::Road(road) => road.should_yield(),
    //         Tile::House(_) => true,
    //         _ => true,
    //     }
    // }
}