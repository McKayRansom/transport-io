use std::rc::{Rc, Weak};

use serde::{Deserialize, Serialize};

use crate::grid::{Id, Position};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reserved {
    #[serde(skip_serializing, skip_deserializing)]
    weak_id: Weak<Id>,
}

impl Reserved {
    pub fn new() -> Self {
        Reserved {
            weak_id: Weak::new(),
        }
    }

    pub fn get_reserved_id(&self) -> Option<Id> {
        Weak::<u64>::upgrade(&self.weak_id).map(|rc| *rc)
    }

    pub fn is_reserved(&self) -> bool {
        self.weak_id.strong_count() > 0
    }

    pub fn try_reserve(&mut self, id: Id, pos: Position) -> Option<Reservation> {
        if !self.is_reserved() {
            let rc = Rc::new(id);
            self.weak_id = Rc::<u64>::downgrade(&rc);
            Some(Reservation {
                _strong_id: rc,
                pos,
            })
        } else {
            None
        }
    }
}

impl PartialEq for Reserved {
    fn eq(&self, other: &Self) -> bool {
        self.weak_id.strong_count() == other.weak_id.strong_count()
    }
}

impl Eq for Reserved {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reservation {
    #[serde(skip_serializing, skip_deserializing)]
    _strong_id: Rc<Id>,
    // we need to be able to re-reserve upon reserializing
    pub pos: Position,
}

impl Reservation {
    pub fn new_for_house(pos: Position) -> Self {
        Reservation {
            _strong_id: Rc::new(0),
            pos,
        }
    }
}

#[cfg(test)]
mod reservation_tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut reserved = Reserved::new();
        assert!(!reserved.is_reserved());

        let pos = Position::new(0, 0);

        let reservation = reserved.try_reserve(1234, pos).unwrap();
        assert!(reserved.is_reserved());
        assert_eq!(reserved.get_reserved_id().unwrap(), 1234);

        assert_eq!(Rc::<u64>::into_inner(reservation._strong_id).unwrap(), 1234);
        assert_eq!(reservation.pos, pos);

        // drop(reservation);

        // assume it is dropped
        assert!(!reserved.is_reserved());
    }
}
