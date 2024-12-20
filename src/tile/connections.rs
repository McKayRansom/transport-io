use std::fmt;

use serde::{Deserialize, Serialize};

use crate::grid::Direction;

pub enum ConnectionLayer {
    Road = 0,
    Driveway = 1,
    Up = 2,
    Down = 3,
}

const CONNECTIONS_ALL: u32 = 0b1111;

#[derive(Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct Connections {
    connection_bitfield: u32,
}

const LAYER_SIZE: u32 = 4;
const LAYER_MASK: u32 = 0b1111;

impl Connections {
    pub fn new() -> Connections {
        Connections {
            connection_bitfield: 0,
        }
    }

    pub fn add(&mut self, layer: ConnectionLayer, dir: Direction) {
        self.connection_bitfield |= (dir as u32) << (layer as u32 * LAYER_SIZE);
    }

    pub fn remove(&mut self, dir: Direction) {
        self.connection_bitfield &= !(dir as u32);
    }

    pub fn count(&self) -> u32 {
        (self.connection_bitfield & LAYER_MASK).count_ones()
    }

    pub fn iter_layer(&self, layer: ConnectionLayer) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: (self.connection_bitfield >> (layer as u32 * LAYER_SIZE))
                & LAYER_MASK,
        }
    }

    pub fn iter(&self) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: self.connection_bitfield,
        }
    }

    pub fn iter_inverse(&self, layer: ConnectionLayer) -> ConnectionsIterator {
        ConnectionsIterator {
            connection_bitfield: (!self.connection_bitfield
                & LAYER_MASK << (LAYER_SIZE & layer as u32)),
        }
    }

    pub fn has(&self, dir: Direction) -> bool {
        (self.connection_bitfield & dir as u32) != 0
    }

    pub fn has_layer(&self, dir: Direction, layer: ConnectionLayer) -> bool {
        ((self.connection_bitfield >> (LAYER_SIZE * layer as u32)) & dir as u32) != 0
    }

}

impl fmt::Debug for Connections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has(Direction::Up) && self.has(Direction::Left) {
            write!(f, "r")
        } else if self.has(Direction::Down) && self.has(Direction::Left) {
            write!(f, "l")
        } else if self.has(Direction::Down) && self.has(Direction::Right) {
            write!(f, "L")
        } else if self.has(Direction::Up) && self.has(Direction::Right) {
            write!(f, "R")
        } else if self.has(Direction::Left) {
            write!(f, "<")
        } else if self.has(Direction::Right) {
            write!(f, ">")
        } else if self.has(Direction::Up) {
            write!(f, "^")
        } else if self.has(Direction::Down) {
            write!(f, ".")
        } else if self.has_layer(Direction::Right, ConnectionLayer::Up) {
            write!(f, "u")
        } else if self.has_layer(Direction::Right, ConnectionLayer::Down) {
            write!(f, "d")
        } else {
            write!(f, "?")
        }
    }
}

pub struct ConnectionsIterator {
    connection_bitfield: u32,
}

impl ConnectionsIterator {
    pub fn all_directions() -> Self {
        ConnectionsIterator {
            connection_bitfield: CONNECTIONS_ALL,
        }
    }

    pub fn no_directions() -> Self {
        ConnectionsIterator {
            connection_bitfield: 0,
        }
    }
}

impl Iterator for ConnectionsIterator {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.connection_bitfield & Direction::Up as u32 != 0 {
            self.connection_bitfield -= Direction::Up as u32;
            Some(Direction::Up)
        } else if self.connection_bitfield & Direction::Down as u32 != 0 {
            self.connection_bitfield -= Direction::Down as u32;
            Some(Direction::Down)
        } else if self.connection_bitfield & Direction::Right as u32 != 0 {
            self.connection_bitfield -= Direction::Right as u32;
            Some(Direction::Right)
        } else if self.connection_bitfield & Direction::Left as u32 != 0 {
            self.connection_bitfield -= Direction::Left as u32;
            Some(Direction::Left)
        } else if self.connection_bitfield != 0 {
            self.connection_bitfield >>= LAYER_SIZE;
            self.next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod connections_tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(Connections::new().count() == 0);
    }

    #[test]
    fn test_iter() {
        let mut connection = Connections::new();
        connection.add(ConnectionLayer::Road, Direction::Right);
        connection.add(ConnectionLayer::Road, Direction::Left);
        assert!(
            connection.iter().collect::<Vec<Direction>>()
                == vec![Direction::Right, Direction::Left]
        );

        // assert!(connection.safe_to_block() == false);
    }

    #[test]
    fn test_layer() {
        let mut connection = Connections::new();
        connection.add(ConnectionLayer::Driveway, Direction::Right);
        connection.add(ConnectionLayer::Road, Direction::Left);
        assert!(
            connection.iter().collect::<Vec<Direction>>()
                == vec![Direction::Left, Direction::Right]
        );
        assert!(
            connection
                .iter_layer(ConnectionLayer::Driveway)
                .collect::<Vec<Direction>>()
                == vec![Direction::Right]
        );

        assert!(connection.count() == 1);
        // assert!(connection.safe_to_block() == true);
    }
}
