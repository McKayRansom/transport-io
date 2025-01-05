use super::{Direction, Map, Position, GRID_CENTER};

impl Map {
    pub fn new_level(level: usize) -> Self {
        let mut map = Self::new();

        match level {
            0 => map.grid.build_two_way_road(GRID_CENTER.into(), Direction::RIGHT).unwrap(),
            _ => {}
        }

        map
    }
}
