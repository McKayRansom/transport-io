use serde::{Deserialize, Serialize};

use crate::{hash_map_id::Id, map::Position};

// #[derive(Clone, Debug, Default)]
pub type Tick = u64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanReservation {
    pub start: Tick,
    pub end: Tick,
    pub pos: Position,
}

impl PlanReservation {
    pub fn new(pos: Position, start: Tick, end: Tick) -> Self {
        Self { start, end, pos }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanReserved {
    id: Id,
    start: Tick,
    end: Tick,
}

impl PlanReserved {
    pub fn create(id: Id, pos: Position, start: Tick, end: Tick) -> (Self, PlanReservation) {
        let reservation = PlanReservation { start, end, pos };
        (
            Self {
                id,
                start,
                end,
            },
            reservation,
        )
    }

    #[allow(unused)]
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
pub struct PlanReservedList {
    list: Vec<PlanReserved>,
}

impl PlanReservedList {
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
        pos: Position,
        cur: Tick,
        start: Tick,
        end: Tick,
    ) -> Option<PlanReservation> {
        let mut to_remove: Option<usize> = None;
        for (i, reserved) in self.list.iter().enumerate() {
            if reserved.is_expired(cur) {
                to_remove = Some(i);
            }
            if reserved.is_reserved(start, end) {
                if reserved.id == id {
                    to_remove = Some(i);
                    break;
                }
                return None;
            }
        }

        let (reserved, reservation) = PlanReserved::create(id, pos, start, end);
        if let Some(i) = to_remove {
            *self.list.get_mut(i).unwrap() = reserved;
        } else {
            self.list.push(reserved);
        }

        Some(reservation)
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

impl From<&[PlanReserved]> for PlanReservedList {
    fn from(value: &[PlanReserved]) -> Self {
        Self { list: value.into() }
    }
}

#[cfg(test)]
mod reservation_tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut reserved = PlanReservedList::new();
        assert!(!reserved.is_reserved(0, 0, 1));

        let pos = Position::new(0, 0);

        let reservation = reserved.try_reserve(1234, pos, 0, 1, 4).unwrap();
        assert_eq!(reservation.pos, pos);

        assert!(!reserved.is_reserved(0, 0, 0));
        assert!(reserved.is_reserved(0, 0, 11));
        assert!(reserved.is_reserved(0, 3, 4));
        assert!(!reserved.is_reserved(0, 5, 5));

        // overlaps start
        assert!(reserved.try_reserve(56, pos, 0, 0, 1).is_none());
        // within
        assert!(reserved.try_reserve(56, pos, 0, 2, 3).is_none());
        // overlaps end
        assert!(reserved.try_reserve(56, pos, 0, 4, 5).is_none());
        // overlaps all
        assert!(reserved.try_reserve(56, pos, 0, 0, 5).is_none());

        let _reservation_later = reserved.try_reserve(1234, pos, 5, 5, 6).unwrap();

        // assume it is dropped
        assert!(!reserved.is_reserved(0, 0, 1));
        assert_eq!(reserved.list.len(), 1);
    }
}
