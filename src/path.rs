use crate::grid::GridPosition as Pos;
use crate::grid::GRID_SIZE;
use crate::grid::Direction;

use pathfinding::prelude::astar;

const DEFAULT_COST: u32 = 2;
const OCCUPIED_COST: u32 = 3;

type GridPathCost = u32;
pub type GridPath = Option<(Vec<Pos>, GridPathCost)>;


pub struct PathTileIter {
    start_pos: Pos,
    connections: u32,
}

impl Iterator for PathTileIter {
    // We can refer to this type using Self::Item
    type Item = Pos;

    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    // We use Self::Item in the return type, so we can change
    // the type without having to update the function signatures.
    fn next(&mut self) -> Option<Self::Item> {
        if self.connections & Direction::Up as u32 != 0 {
            self.connections -= Direction::Up as u32;
            Some(Pos {
                x: self.start_pos.x,
                y: self.start_pos.y - 1,
            })
        } else if self.connections & Direction::Down as u32 != 0 {
            self.connections -= Direction::Down as u32;
            Some(Pos {
                x: self.start_pos.x,
                y: self.start_pos.y + 1,
            })
        } else if self.connections & Direction::Right as u32 != 0 {
            self.connections -= Direction::Right as u32;
            Some(Pos {
                x: self.start_pos.x + 1,
                y: self.start_pos.y,
            })
        } else if self.connections & Direction::Left as u32 != 0 {
            self.connections -= Direction::Left as u32;
            Some(Pos {
                x: self.start_pos.x - 1,
                y: self.start_pos.y,
            })
        }
        else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct PathTile {
    allowed: bool,
    occupied: bool,
    connections: u32,
}

impl PathTile {
    fn new() -> PathTile {
        PathTile {
            allowed: false,
            occupied: false,
            connections: 0,
        }
    }

    fn connect(&mut self, dir: Direction) {
        self.connections |= dir as u32;
    }

    fn is_connected(&self, dir: Direction) -> bool {
        self.connections & (dir as u32) != 0
    }

    fn connections_count(&self) -> u32 {
        self.connections.count_ones()
    }

    fn connections_as_iter(&self, start_pos: Pos) -> PathTileIter {
        PathTileIter {
            connections: self.connections,
            start_pos,
        }
    }
}

pub struct PathGrid {
    tiles: Vec<Vec<PathTile>>,
}

impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }
}

impl PathGrid {
    pub fn new() -> Self {
        PathGrid {
            tiles: vec![vec![PathTile::new(); GRID_SIZE.1 as usize]; GRID_SIZE.0 as usize],
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
        self.tiles[pos.x as usize][pos.y as usize]
            .connections_as_iter(pos)
            // .filter(|x| self.allowed.contains(x) && x.valid())
            .map(|p| {
                (
                    p,
                    if self.is_occupied(p) {
                        OCCUPIED_COST
                    } else {
                        DEFAULT_COST
                    },
                )
            })
            .collect()
    }

    pub fn connection_count(&self, pos: Pos) -> u32 {
        self.tiles[pos.x as usize][pos.y as usize].connections_count()
    }

    pub fn is_allowed(&self, pos: Pos) -> bool {
        self.tiles[pos.x as usize][pos.y as usize].allowed
    }

    pub fn get_dirs(&self, pos: Pos) -> PathTileIter {
        self.tiles[pos.x as usize][pos.y as usize].connections_as_iter(pos)
    }

    pub fn add_allowed(&mut self, pos: Pos, direction: Direction) {
        self.tiles[pos.x as usize][pos.y as usize].allowed = true;
        self.tiles[pos.x as usize][pos.y as usize].connect(direction);
    }

    pub fn remove_allowed(&mut self, pos: Pos) {
        self.tiles[pos.x as usize][pos.y as usize].allowed = false;
        self.tiles[pos.x as usize][pos.y as usize].connections = 0;
    }

    pub fn is_occupied(&self, pos: Pos) -> bool {
        self.tiles[pos.x as usize][pos.y as usize].occupied
    }

    pub fn add_occupied(&mut self, pos: Pos) {
        self.tiles[pos.x as usize][pos.y as usize].occupied = true
    }

    pub fn remove_occupied(&mut self, pos: Pos) {
        self.tiles[pos.x as usize][pos.y as usize].occupied = false
    }
}
