
use crate::grid::GridPosition as Pos;

use pathfinding::prelude::astar;

const GRID_COST: u32 = 1;

type GridPathCost = u32;
pub type GridPath = Option<(Vec<Pos>, GridPathCost)>;

pub struct PathGrid {
    occupied: Vec<Pos>,
}


impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }

    fn successors(&self, blocked: &Vec<Pos>) -> Vec<(Pos, u32)> {
        vec![
            Pos {
                x: self.x + 1,
                y: self.y,
            },
            Pos {
                x: self.x - 1,
                y: self.y,
            },
            Pos {
                x: self.x,
                y: self.y + 1,
            },
            Pos {
                x: self.x,
                y: self.y - 1,
            },
        ]
        .into_iter()
        .filter(|x| !blocked.contains(x))
        .map(|p| (p, GRID_COST))
        .collect()
    }
}

impl PathGrid {

    pub fn new() -> Self {
        PathGrid {
            occupied: Vec::new(),
        }
    }

    pub fn find_path(&self, start: Pos, end: Pos) -> GridPath {
        let result = astar(
            &start,
            |p| p.successors(&self.occupied),
            |p| p.distance(&end) / 3,
            |p| *p == end,
        );
        // assert_eq!(result.expect("no path found").1, 4);

        result
    }

    pub fn add_blocked(&mut self, pos: Pos) {
        self.occupied.push(pos);
    }

    pub fn remove_blocked(&mut self, pos: Pos) {
        if let Some(index) = self.occupied.iter().position(|value| *value == pos) {
            self.occupied.swap_remove(index);
        }
    }
}
