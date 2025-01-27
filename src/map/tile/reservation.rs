use serde::{Deserialize, Serialize};

use crate::{hash_map_id::Id, map::Position};

// #[derive(Clone, Debug, Default)]
pub type Tick = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanReservation {
    pub start: Tick,
    pub end: Tick,
    pub pos: Position,
}

impl PlanReservation {
    pub fn new_for_building(pos: Position, start: Tick, end: Tick) -> Self {
        Self {start, end, pos}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanReserved {
    id: Id,
    start_tick: Tick,
    end_tick: Tick,
}

impl PlanReserved {
    pub fn create(id: Id, pos: Position, start: Tick, end: Tick) -> (Self, PlanReservation) {
        let reservation = PlanReservation {
            start,
            end,
            pos,
        };
        (
            Self {
                id,
                start_tick: start,
                end_tick: end,
            },
            reservation,
        )
    }

    pub fn get_reserved_id(&self) -> Id {
        self.id
    }

    pub fn is_reserved(&self, tick: Tick) -> bool {
        self.start_tick <= tick && self.end_tick >= tick
    }

    pub fn is_reserved_duration(&self, start: Tick, end: Tick) -> bool {
        (self.start_tick <= start && self.end_tick >= start) || (self.start_tick <= end && self.end_tick >= end) || (
            start < self.start_tick && end > self.end_tick
        )
    }

    pub fn is_expired(&self, tick: Tick) -> bool {
        self.end_tick < tick
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

    pub fn get_reserved_id(&self, tick: Tick) -> Option<Id> {
        for reserved in self.list.iter() {
            if reserved.is_reserved(tick) {
                return Some(reserved.get_reserved_id());
            }
        }
        None
    }

    pub fn is_reserved(&self, tick: Tick) -> bool {
        self.get_reserved_id(tick).is_some()
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
            if reserved.is_reserved_duration(start, end) {
                return None;
            }
        }

        let (reserved, reservation) = PlanReserved::create(id, pos, start, end);
        if let Some(i) = to_remove {
            *self.list.get_mut(i).unwrap() = reserved;
        } else {
            self.list.push(reserved);
        }
        return Some(reservation);
    }
}


#[cfg(test)]
mod reservation_tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut reserved = PlanReservedList::new();
        assert!(!reserved.is_reserved(0));

        let pos = Position::new(0, 0);

        let reservation = reserved.try_reserve(1234, pos, 0, 1, 4).unwrap();
        assert_eq!(reservation.pos, pos);

        assert_eq!(reserved.is_reserved(0), false);
        assert_eq!(reserved.is_reserved(1), true);
        assert_eq!(reserved.is_reserved(2), true);
        assert_eq!(reserved.is_reserved(3), true);
        assert_eq!(reserved.is_reserved(4), true);
        assert_eq!(reserved.is_reserved(5), false);

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
        assert_eq!(reserved.is_reserved(1), false);
        assert_eq!(reserved.list.len(), 1);
    }
}
