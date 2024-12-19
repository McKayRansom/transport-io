
use std::rc::{Rc, Weak};

use crate::grid::Id;

#[derive(Clone, Debug)]
pub struct Reserved {
    weak_id: Weak<Id>,
}

impl Reserved {
    pub fn new() -> Self {
        Reserved{
            weak_id: Weak::new()
        }
    }

    pub fn get_reserved_id(&self) -> Option<Id> {
        if let Some(rc) = Weak::<u64>::upgrade(&self.weak_id) {
            Some(*rc)
        } else {
            None
        }
    }

    pub fn is_reserved(&self) -> bool {
        self.weak_id.strong_count() > 0
    }

    pub fn try_reserve(&mut self, id: Id) -> Option<Reservation> {
        if !self.is_reserved() {
            let rc = Rc::new(id);
            self.weak_id = Rc::<u64>::downgrade(&rc);
            Some(Reservation{
                strong_id: rc
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

#[derive(Clone, Debug)]
pub struct Reservation {
    strong_id: Rc<Id>,
}

impl Reservation {
    pub fn new_for_house() -> Self {
        Reservation {
            strong_id: Rc::new(0)
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

        let reservation = reserved.try_reserve(1234).unwrap();
        assert!(reserved.is_reserved());
        assert_eq!(reserved.get_reserved_id().unwrap(), 1234);

        assert_eq!(Rc::<u64>::into_inner(reservation.strong_id).unwrap(), 1234);

        // drop(reservation);

        // assume it is dropped
        assert!(!reserved.is_reserved());
    }

}
