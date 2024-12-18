use std::fmt;

use super::Direction;


pub enum ConnectionLayer {
    Road = 0,
    Driveway = 1,
    // Bridge = 2,
}

const CONNECTIONS_ALL: u32 = 0b1111;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Connections {
    connection_bitfield: u32,
}

const LAYER_SIZE: u32 = 4;
const LAYER_MASK: u32 = 0b1111;

impl Connections {
    pub fn new(layer: ConnectionLayer, dir: Direction) -> Connections {
        Connections {
            connection_bitfield: (dir as u32) << ((layer as u32) * LAYER_SIZE),
        }
    }

    pub fn add(&mut self, layer: ConnectionLayer, dir: Direction) {
        self.connection_bitfield |= (dir as u32) << layer as u32 * LAYER_SIZE;
    }

    pub fn remove(&mut self, dir: Direction) {
        self.connection_bitfield &= !(dir as u32);
    }

    pub fn count(&self) -> u32 {
        (self.connection_bitfield & LAYER_MASK).count_ones()
    }

    // pub fn safe_to_block(&self) -> bool {
    //     // Don't block intersections!
    //     // but only for real road intersections
    //     self.count() < 2
    //     // true
    // }

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
            connection_bitfield: (!self.connection_bitfield & LAYER_MASK << (LAYER_SIZE & layer as u32)),
        }
    }

    pub fn has(&self, dir: Direction) -> bool {
        (self.connection_bitfield & dir as u32) != 0
    }
}

impl fmt::Debug for Connections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.connection_bitfield == Direction::Left as u32 {
            write!(f, "<")
        }
        else if self.connection_bitfield == Direction::Right as u32 {
            write!(f, ">")
        }
        else if self.connection_bitfield == Direction::Up as u32 {
            write!(f, "^")
        }
        else if self.connection_bitfield == Direction::Down as u32 {
            write!(f, ".")
        }
        else if self.connection_bitfield == (Direction::Up as u32 | Direction::Left as u32) {
            write!(f, "r")
        }
        else if self.connection_bitfield == (Direction::Down as u32 | Direction::Left as u32) {
            write!(f, "l")
        }
        else if self.connection_bitfield == (Direction::Down as u32 | Direction::Right as u32) {
            write!(f, "L")
        }
        else if self.connection_bitfield == (Direction::Up as u32 | Direction::Right as u32) {
            write!(f, "R")
        }
        else {
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
            self.connection_bitfield = self.connection_bitfield >> LAYER_SIZE;
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
        assert!(Connections::new(ConnectionLayer::Road, Direction::Right).count() == 1);
    }

    #[test]
    fn test_iter() {
        let mut connection = Connections::new(ConnectionLayer::Road, Direction::Right);
        connection.add(ConnectionLayer::Road, Direction::Left);
        assert!(
            connection.iter().collect::<Vec<Direction>>()
                == vec![Direction::Right, Direction::Left]
        );

        // assert!(connection.safe_to_block() == false);
    }

    #[test]
    fn test_layer() {
        let mut connection = Connections::new(ConnectionLayer::Driveway, Direction::Right);
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
