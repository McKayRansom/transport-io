use serde::{Deserialize, Serialize};

use crate::{
    hash_map_id::Id,
    map::{grid::Grid, Position},
};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReservationError {
    TileInvalid,
    TileReserved,
}

// #[derive(Clone, Debug, Default)]
pub type Tick = u32;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reservation {
    pub start: Tick,
    pub end: Tick,
    pub pos: Position,
}

impl Reservation {
    pub fn new(pos: Position, start: Tick, end: Tick) -> Self {
        Self { start, end, pos }
    }

    pub fn reserve(&self, grid: &mut Grid, vehicle_id: Id, tick: Tick)-> Result<(), ReservationError> {
        grid.get_tile_mut(&self.pos)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(vehicle_id, tick, self)
    }

    #[allow(unused)]
    pub fn unreserve(&self, grid: &mut Grid, vehicle_id: Id) -> Result<(), ReservationError> {
        grid.get_tile_mut(&self.pos)
            .ok_or(ReservationError::TileInvalid)?
            .unreserve(vehicle_id);

        Ok(())
    }

    pub fn is_reserved(
        &self,
        grid: &mut Grid, 
        vehicle_id: Id
    ) -> Result<(), ReservationError> {
        grid.get_tile(&self.pos)
            .ok_or(ReservationError::TileInvalid)?
            .is_reserved(vehicle_id, self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reserved {
    id: Id,
    start: Tick,
    end: Tick,
}

impl Reserved {
    pub fn new(id: Id, start: Tick, end: Tick) -> Self {
        Self { id, start, end }
    }

    pub fn get_reserved_id(&self) -> Id {
        self.id
    }

    pub fn is_reserved(&self, start: Tick, end: Tick) -> bool {
        (self.start <= start && self.end >= start)
            || (self.start <= end && self.end >= end)
            || (start < self.start && end > self.end)
    }

    pub fn is_expired(&self, tick: Tick) -> bool {
        self.end < tick || self.id == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservedList {
    list: Vec<Reserved>,
}

impl ReservedList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn get_reserved_id(&self, start: Tick, end: Tick) -> Option<Id> {
        for reserved in self.list.iter() {
            if reserved.is_reserved(start, end) {
                return Some(reserved.get_reserved_id());
            }
        }
        None
    }

    pub fn is_reserved(&self, res_id: Id, start: Tick, end: Tick) -> bool {
        match self.get_reserved_id(start, end) {
            Some(id) => res_id != id,
            None => false,
        }
    }

    pub fn try_reserve(
        &mut self,
        id: Id,
        cur: Tick,
        reservation: &Reservation,
    ) -> Result<(), ReservationError> {
        let mut to_remove: Option<usize> = None;
        for (i, reserved) in self.list.iter().enumerate() {
            if reserved.is_expired(cur) {
                to_remove = Some(i);
            }
            if reserved.is_reserved(reservation.start, reservation.end) {
                if reserved.id == id {
                    to_remove = Some(i);
                    break;
                }
                return Err(ReservationError::TileReserved);
            }
        }

        let reserved = Reserved::new(id, reservation.start, reservation.end);
        if let Some(i) = to_remove {
            *self.list.get_mut(i).unwrap() = reserved;
        } else {
            self.list.push(reserved);
        }

        Ok(())
    }

    pub fn unreserve(&mut self, id: Id) {
        for res in self.list.iter_mut() {
            if res.id == id {
                // mark to be reused later
                res.start = 0;
                res.end = 0;
                res.id = 0;
            }
        }
    }
}

impl From<&[Reserved]> for ReservedList {
    fn from(value: &[Reserved]) -> Self {
        Self { list: value.into() }
    }
}

#[cfg(test)]
mod reservation_tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut reserved = ReservedList::new();
        assert!(!reserved.is_reserved(0, 0, 1));

        let pos = Position::new(0, 0);

        reserved
            .try_reserve(1234, 0, &Reservation::new(pos, 1, 4))
            .unwrap();

        assert!(!reserved.is_reserved(0, 0, 0));
        assert!(reserved.is_reserved(0, 0, 11));
        assert!(reserved.is_reserved(0, 3, 4));
        assert!(!reserved.is_reserved(0, 5, 5));

        // overlaps start
        assert!(reserved
            .try_reserve(56, 0, &Reservation::new(pos, 0, 1))
            .is_err());
        // within
        assert!(reserved
            .try_reserve(56, 0, &Reservation::new(pos, 2, 3))
            .is_err());
        // overlaps end
        assert!(reserved
            .try_reserve(56, 0, &Reservation::new(pos, 4, 5))
            .is_err());
        // overlaps all
        assert!(reserved
            .try_reserve(56, 0, &Reservation::new(pos, 0, 5))
            .is_err());

        let _reservation_later = reserved
            .try_reserve(1234, 5, &Reservation::new(pos, 5, 6))
            .unwrap();

        // assume it is dropped
        assert!(!reserved.is_reserved(0, 0, 1));
        assert_eq!(reserved.list.len(), 1);
    }

    #[test]
    fn test_tick_duration() {
        let tick_max = Tick::MAX;

        let tick_secs = tick_max / 60;
        let tick_mins = tick_secs / 60;
        let tick_hours = tick_mins / 60;

        let tick_days = tick_hours / 24;
        let tick_years = tick_days / 365;

        assert_eq!(tick_years, 2);
    }
}
