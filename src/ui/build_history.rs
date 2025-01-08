use std::collections::VecDeque;

use crate::map::{
    build::{BuildAction, BuildResult},
    Map,
};

const BUILD_HISTORY_MAX: usize = 16;

pub struct BuildHistory {
    queue: VecDeque<Box<dyn BuildAction>>,
    undo_pos: usize,
}

impl BuildHistory {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            undo_pos: 0,
        }
    }

    pub fn do_action(&mut self, map: &mut Map, mut action: Box<dyn BuildAction>) -> BuildResult {
        // don't add if it fails!
        action.execute(map)?;
        self.queue.push_front(action);
        if self.queue.len() > BUILD_HISTORY_MAX {
            self.queue.pop_back();
        }
        self.undo_pos = 0;

        Ok(())
    }

    pub fn undo_action(&mut self, map: &mut Map) -> BuildResult {
        if let Some(item) = self.queue.get_mut(self.undo_pos) {
            item.undo(map)?;
            self.undo_pos += 1;
        }

        Ok(())
    }

    pub fn redo_action(&mut self, map: &mut Map) -> BuildResult {
        if self.undo_pos == 0 {
            return Ok(());
        }
        self.undo_pos -= 1;
        if let Some(item) = self.queue.get_mut(self.undo_pos) {
            item.execute(map)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod build_history_tests {
    use crate::map::{build::BuildRoad, grid::Grid, Direction};

    use super::*;

    #[test]
    fn test_undo() {
        let mut map = Map::new_blank((4, 1));
        let mut build_history = BuildHistory::new();

        build_history
            .do_action(
                &mut map,
                Box::new(BuildRoad::new((0, 0).into(), Direction::RIGHT)),
            )
            .unwrap();

            build_history
            .do_action(
                &mut map,
                Box::new(BuildRoad::new((1, 0).into(), Direction::RIGHT)),
            )
            .unwrap();

            build_history
            .do_action(
                &mut map,
                Box::new(BuildRoad::new((2, 0).into(), Direction::RIGHT)),
            )
            .unwrap();

        assert_eq!(map.grid, Grid::new_from_string(">>>e"));
        build_history.undo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>ee"));
        build_history.undo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">eee"));
        build_history.undo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("eeee"));
        // Error here?
        build_history.undo_action(&mut map).unwrap();

        build_history.redo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">eee"));
        build_history.redo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>ee"));
        build_history.redo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>>e"));

        build_history.undo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>ee"));
        build_history
            .do_action(
                &mut map,
                Box::new(BuildRoad::new((0, 0).into(), Direction::RIGHT)),
            )
            .unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>ee"));
        // redo fails
        build_history.redo_action(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>ee"));
    }
}
