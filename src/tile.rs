use macroquad::{
    color::{Color, WHITE},
    math::Rect,
};

mod road;
pub use road::*;

mod connections;
pub use connections::*;

use crate::{
    grid::{Id, Position, ReservationStatus},
    tileset::Tileset,
};

const HOUSE_SPRITE: u32 = (16 * 1) + 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct House {
    pub vehicle_on_the_way: Option<Id>,
}

impl House {
    pub fn new() -> Self {
        House { vehicle_on_the_way: None }
    }

    pub fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let color = if self.vehicle_on_the_way.is_some() {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        tileset.draw_tile(HOUSE_SPRITE, color, &rect, 0.0);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    House(House),
    Road(Road),
}

impl Tile {
    pub fn new() -> Self {
        Tile::Empty
    }

    pub fn new_from_char(ch: char) -> Self {
        match ch {
            'h' => Tile::House(House {
                vehicle_on_the_way: None,
            }),
            '_' => Tile::Empty,
            _ => {
                if let Some(road) = Road::new_from_char(ch) {
                    Tile::Road(road)
                } else {
                    Tile::Empty
                }
            }
        }
    }

    pub fn iter_connections(&self) -> ConnectionsIterator {
        match self {
            Tile::Road(road) => road.iter_connections(),
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

    pub fn draw_bridge(&self, pos: Position, tileset: &Tileset) {
        let mut rect = Rect::from(pos);
        rect.y -= 10.;
        match self {
            Tile::Road(road) => road.draw(&rect, tileset),
            _ => {}
        }
    }


    pub fn reserve(&mut self, id: Id) -> ReservationStatus {
        match self {
            Tile::Road(road) => {
                if road.reserved.is_some()
                /* TODO: Add check for intersection full */
                {
                    ReservationStatus::TileReserved
                // } else if road.connections.safe_to_block() {
                // road.reserved = true;
                // ReservationStatus::TileBlockable
                } else {
                    road.reserved = Some(id);
                    ReservationStatus::TileSuccess
                    // ReservationStatus::TileDoNotBlock
                }
            }
            Tile::House(_) => ReservationStatus::TileSuccess,
            Tile::Empty => ReservationStatus::TileInvalid,
        }
    }

    pub fn unreserve(&mut self) {
        if let Tile::Road(road) = self {
            road.reserved = None;
        }
    }

    pub fn clear(&mut self) {
        *self = Tile::Empty;
    }

    pub fn build<F>(&mut self, func: F)
    where
        F: FnOnce() -> Tile,
    {
        if *self == Tile::Empty {
            *self = func()
        }
    }

    pub fn edit_road<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Road),
    {
        match self {
            Tile::Empty => {
                let mut road = Road::new();
                func(&mut road);
                *self = Tile::Road(road);
            }
            Tile::Road(road) => {
                func(road);
            }
            _ => {}
        }
    }

    // pub fn add(&mut self, other: &Tile) {
    //     match self {
    //         Tile::Road(road) => {road.add(other) },
    //         Tile::Empty => { *self = *other },
    //     }
    // }

    // pub fn should_yield(&self) -> bool {
    //     match self {
    //         Tile::Road(road) => road.should_yield(),
    //         Tile::House(_) => true,
    //         _ => true,
    //     }
    // }
}
