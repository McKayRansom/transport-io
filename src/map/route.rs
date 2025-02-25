use serde::{Deserialize, Serialize};

use crate::hash_map_id::Id;



#[derive(Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Route {
    stops: Vec<Id>,
    // is_loop: bool,
}

impl Route {

    pub fn add_stop(&mut self, id: Id, index: usize) {
        self.stops.insert(index, id);
    }

    pub fn get_stop(&mut self, index: usize) -> Option<Id> {
        self.stops.get(index).cloned()
    }

}


mod route_tests {

    use super::*;

    #[test]
    fn test_init() {
        let mut route = Route::default();

        route.add_stop(1234, 0);

        assert_eq!(route.get_stop(0), Some(1234));
        assert_eq!(route.get_stop(1), None);
    }
}




 