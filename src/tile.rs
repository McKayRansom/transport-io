use macroquad::{
    color::{Color, WHITE},
    math::Rect,
};

mod road;
pub use road::*;

mod connections;
pub use connections::*;

mod reservation;
pub use reservation::*;
use serde::{Deserialize, Serialize};

use crate::{
    grid::{Direction, Id, Position, ReservationError},
    tileset::Tileset,
};

const HOUSE_SPRITE: u32 = 16;

const DEFAULT_COST: u32 = 1;
const OCCUPIED_COST: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum YieldType {
    Always,
    IfAtIntersection,
    Never,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[derive(Serialize, Deserialize)]
pub struct House {
    pub vehicle_on_the_way: Option<Id>,
}

impl House {
    pub fn new() -> Self {
        House {
            vehicle_on_the_way: None,
        }
    }

    pub fn draw(&self, rect: &Rect, tileset: &Tileset) {
        let color = if self.vehicle_on_the_way.is_some() {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        tileset.draw_tile(HOUSE_SPRITE, color, rect, 0.0);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[derive(Serialize, Deserialize)]
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

        if let Tile::Road(road) = self {
            road.draw(&rect, tileset);
        }
    }

    pub fn draw_bridge(&self, pos: Position, tileset: &Tileset) {
        let mut rect = Rect::from(pos);
        rect.y -= 10.;
        if let Tile::Road(road) = self {
            road.draw(&rect, tileset);
        }
    }

    pub fn reserve(&mut self, id: Id, pos: Position) -> Result<Reservation, ReservationError> {
        match self {
            Tile::Road(road) => road
                .reserved
                .try_reserve(id, pos)
                .ok_or(ReservationError::TileReserved),

            Tile::House(_) => Ok(Reservation::new_for_house(pos)),
            Tile::Empty => Err(ReservationError::TileInvalid),
        }
    }

    pub fn should_yield(&self) -> YieldType {
        match self {
            // alway yield from a house
            Tile::House(_) => YieldType::Always,
            // if we are somehow in a weird state, I guess yield?
            Tile::Empty => YieldType::Always,
            Tile::Road(road) => {
                if road.connection_count() > 1 {
                    YieldType::Never
                } else {
                    YieldType::IfAtIntersection
                }
            }
        }
    }

    pub fn should_be_yielded_to(&self, should_yield: YieldType, dir_from: Direction) -> bool {
        if let Tile::Road(road) = self {
            if road.reserved.is_reserved() && road.is_connected(dir_from.inverse()) {
                if should_yield == YieldType::Always {
                    true
                } else if road.connection_count() > 1 {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            Tile::Road(road) => {
                if road.reserved.is_reserved() {
                    OCCUPIED_COST
                } else {
                    DEFAULT_COST
                }
            }
            Tile::House(_) => DEFAULT_COST * 2,
            Tile::Empty => DEFAULT_COST * 3,
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
