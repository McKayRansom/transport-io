use std::fmt;

use macroquad::{
    color::{Color, WHITE},
    math::Rect,
};

mod road;
pub use road::*;

mod reservation;
pub use reservation::*;
use serde::{Deserialize, Serialize};

use crate::{
    grid::{Direction, Id, Position, ReservationError},
    tileset::{Sprite, Tileset},
};

const HOUSE_SPRITE: Sprite = Sprite::new_size(1, 0, (1, 1));

const DEFAULT_COST: u32 = 1;
const OCCUPIED_COST: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum YieldType {
    Always,
    IfAtIntersection,
    Never,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Building {
    pub vehicle_on_the_way: Option<Id>,
}

impl Building {
    pub fn new() -> Self {
        Building {
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
    Building(Building),
    Road(Road),
    Ramp(Ramp),
}

impl Tile {
    pub fn new() -> Self {
        Tile::Empty
    }

    pub fn new_from_char(ch: char) -> Self {
        match ch {
            'h' => Tile::Building(Building {
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

    pub fn iter_connections(&self) -> std::slice::Iter<'_, Direction> {
        match self {
            Tile::Road(road) => road.iter_connections(),
            Tile::Building(_) => Direction::ALL.iter(),
            _ => [].iter(),
        }
    }

    #[allow(clippy::single_match)]
    pub fn draw(&self, pos: Position, tileset: &Tileset) {
        let rect = Rect::from(pos);

        match self {
            Tile::Road(road) => road.draw(&rect, tileset),
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

    pub fn road_successor(&self, pos: &Position) -> (Position, u32) {
        (
            match self {
                Tile::Ramp(ramp) => *pos + ramp.dir,
                _ => *pos,
            },
            // if self.is_road() {
            //     new_pos
            // } else {
            //     *pos + dir.rotate_left()
            // },
            self.cost(),
        )
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
