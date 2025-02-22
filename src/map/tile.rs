
mod road;
pub use road::*;

mod reservation;
pub use reservation::*;

use std::fmt;
use serde::{Deserialize, Serialize};
use crate::hash_map_id::Id;
use super::Direction;

const DEFAULT_COST: u32 = 1;
const _OCCUPIED_COST: u32 = 2;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Building(Id),
    Road(Road),
    Water,
}

impl Tile {
    pub fn new() -> Self {
        Tile::Empty
    }


    // TODO: Inline this into Reservation
    pub fn reserve(
        &mut self,
        id: Id,
        current: Tick,
        reservation: &Reservation,
    ) -> Result<(), ReservationError> {
        // TEST: allow vehicles to cross in intersections...
        // if !self.is_blockable() {
        //     return Ok(());
        // }
        match self {
            Tile::Road(road) => road.try_reserve(id, current, reservation),

            Tile::Building(_) => Ok(()),
            _ => Err(ReservationError::TileInvalid),
        }
    }

    #[allow(clippy::single_match)]
    pub fn unreserve(&mut self, id: Id, pos: RoadPosition) {
        match self {
            Tile::Road(road) => road.unreserve(id, pos),

            _ => {}
        }
    }

    pub fn is_reserved(&self, id: Id, pos: RoadPosition, start: Tick, end: Tick) -> Result<(), ReservationError> {
        match self {
            Tile::Road(road) => {
                if road.is_reserved(id, pos, start, end) {
                    Err(ReservationError::TileReserved)
                } else {
                    Ok(())
                }
            }
            Tile::Building(_) => Ok(()),
            // T
            _ => Err(ReservationError::TileInvalid),
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            // TODO: THINK ABOUT THIS COST!
            Tile::Road(_road) => DEFAULT_COST, // road.reserved.cost(),
            Tile::Building(_) => DEFAULT_COST * 2,
            // we run into this for dead-end turn around
            _ => DEFAULT_COST,
        }
    }

    pub fn is_road(&self) -> bool {
        matches!(self, Tile::Road(_))
    }

    pub fn is_blockable(&self) -> bool {
        if let Tile::Road(road) = self {
            road.connections.count() <= 2
        } else {
            true
        }
    }

    pub fn get_building_id(&self) -> Option<Id> {
        match self {
            Tile::Building(building_id) => Some(*building_id),
            Tile::Road(road) => road.station,
            _ => None,
        }
    }
}

impl From<char> for Tile {
    fn from(ch: char) -> Self {
        match ch {
            'h' => Tile::Building(0),
            '_' => Tile::Empty,
            'w' => Tile::Water,
            '1'..'9' => Tile::Road(Road::new_connected(
                Direction::NONE,
                Some(ch as Id - '0' as Id),
            )),
            _ => {
                if let Ok(road) = ch.try_into() {
                    Tile::Road(road)
                } else {
                    Tile::Empty
                }
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
            Tile::Water => write!(f, "w"),
        }
    }
}
