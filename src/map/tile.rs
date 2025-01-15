use std::fmt;

use macroquad::math::Rect;

mod road;
pub use road::*;

mod reservation;
pub use reservation::*;
use serde::{Deserialize, Serialize};

use crate::{
    tileset::Tileset, hash_map_id::Id,
};

use super::{grid::ReservationError, Direction, Position};

const DEFAULT_COST: u32 = 1;
const OCCUPIED_COST: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum YieldType {
    Always,
    IfAtIntersection,
    Never,
}


#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ramp {
    dir: Direction,
}

impl Ramp {
    pub fn new(dir: Direction) -> Self {
        Self { dir }
    }
}

impl fmt::Debug for Ramp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dir {
            Direction::LAYER_UP  => write!(f, "u"),
            Direction::LAYER_DOWN => write!(f, "d"),
            _ => write!(f, "r?"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Building(Id),
    Road(Road),
    Ramp(Ramp),
}

impl Tile {
    pub fn new() -> Self {
        Tile::Empty
    }

    pub fn new_from_char(ch: char) -> Self {
        match ch {
            'h' => Tile::Building(0),
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

    pub fn iter_connections(&self, pos: &Position) -> &[Direction] {
        match self {
            Tile::Road(road) => road.iter_connections(pos),
            // we gotta fix this
            Tile::Building(_) => pos.default_connections(),
            _ => [].as_slice(),
        }
    }

    #[allow(clippy::single_match)]
    pub fn draw(&self, pos: Position, tileset: &Tileset) {
        let rect = Rect::from(pos);

        match self {
            Tile::Road(road) => road.draw(pos, &rect, tileset),
            // Tile::Empty => tileset.draw_rect(&rect, LIGHTGRAY),
            _ => {}
        }
    }

    pub fn draw_bridge(&self, pos: Position, tileset: &Tileset, tile_below: &Tile) {
        if let Tile::Road(road) = self {
            if let Tile::Ramp(_) = tile_below {
                road.draw_bridge(&pos, tileset, true);
            } else {
                road.draw_bridge(&pos, tileset, false);
            }
        }
    }

    pub fn reserve(&mut self, id: Id, pos: Position) -> Result<Reservation, ReservationError> {
        match self {
            Tile::Road(road) => road
                .reserved
                .try_reserve(id, pos)
                .ok_or(ReservationError::TileReserved),

            Tile::Building(_) => Ok(Reservation::new_for_building(pos)),
            _ => Err(ReservationError::TileInvalid),
        }
    }

    pub fn should_yield(&self) -> YieldType {
        match self {
            Tile::Road(road) => {
                if road.connection_count() > 1 {
                    YieldType::Never
                } else {
                    YieldType::IfAtIntersection
                }
            }

            // alway yield from a building
            // if we are somehow in a weird state, I guess yield?
            _ => YieldType::Always,
        }
    }

    pub fn should_be_yielded_to(&self, should_yield: YieldType, dir_from: Direction) -> bool {
        if let Tile::Road(road) = self {
            if road.reserved.is_reserved() && road.is_connected(dir_from.inverse()) {
                should_yield == YieldType::Always || road.connection_count() > 1
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
            Tile::Building(_) => DEFAULT_COST * 2,
            // we run into this for dead-end turn around
            _ => DEFAULT_COST * 3,
        }
    }

    pub(crate) fn is_road(&self) -> bool {
        matches!(self, Tile::Road(_))
    }

    pub fn get_building_id(&self) -> Option<Id> {
        match self {
            Tile::Building(building_id) => {
                Some(*building_id)
            }
            Tile::Road(road) => {
                road.station
            }
            _ => {
                None
            }
        }
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Empty => write!(f, "e"),
            Tile::Road(road) => road.fmt(f),
            Tile::Building(_) => write!(f, "h"),
            Tile::Ramp(ramp) => ramp.fmt(f),
            // => write!(f, "b")?,
        }
    }
}
