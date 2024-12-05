use crate::grid::GridPosition as Pos;

use pathfinding::prelude::astar;

const DEFAULT_COST: u32 = 1;
const OCCUPIED_COST: u32 = 2;

type GridPathCost = u32;
pub type GridPath = Option<(Vec<Pos>, GridPathCost)>;

pub struct PathGrid {
    allowed: Vec<Pos>,
    occupied: Vec<Pos>,
}

impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }
}

impl PathGrid {
    pub fn new() -> Self {
        PathGrid {
            allowed: Vec::new(),
            occupied: Vec::new(),
        }
    }

    pub fn find_path(&self, start: Pos, end: Pos) -> GridPath {
        let result = astar(
            &start,
            |p| self.successors(*p),
            |p| p.distance(&end) / 3,
            |p| *p == end,
        );
        // assert_eq!(result.expect("no path found").1, 4);

        result
    }

    fn successors(&self, pos: Pos) -> Vec<(Pos, u32)> {
        vec![
            Pos {
                x: pos.x + 1,
                y: pos.y,
            },
            Pos {
                x: pos.x - 1,
                y: pos.y,
            },
            Pos {
                x: pos.x,
                y: pos.y + 1,
            },
            Pos {
                x: pos.x,
                y: pos.y - 1,
            },
        ]
        .into_iter()
        .filter(|x| self.allowed.contains(x) && x.valid())
        .map(|p| {
            (
                p,
                if self.occupied.contains(&p) {
                    OCCUPIED_COST
                } else {
                    DEFAULT_COST
                },
            )
        })
        .collect()
    }

    pub fn is_allowed(&self, pos: Pos) -> bool {
        self.allowed.contains(&pos)
    }

    pub fn add_allowed(&mut self, pos: Pos) {
        self.allowed.push(pos);
    }

    pub fn remove_allowed(&mut self, pos: Pos) {
        if let Some(index) = self.allowed.iter().position(|value| *value == pos) {
            self.allowed.swap_remove(index);
        }
    }

    pub fn is_occupied(&self, pos: Pos) -> bool {
        self.occupied.contains(&pos)
    }

    pub fn add_occupied(&mut self, pos: Pos) {
        self.occupied.push(pos);
    }

    pub fn remove_occupied(&mut self, pos: Pos) {
        if let Some(index) = self.occupied.iter().position(|value| *value == pos) {
            self.occupied.swap_remove(index);
        }
    }
}
