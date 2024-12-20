use std::{
    fmt,
    iter::{Enumerate, FilterMap},
};

use serde::{Deserialize, Serialize};

use crate::grid::Direction;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectionType {
    #[default]
    None = 0,
    Road = 1,
    Up = 2,
    Down = 3,
}

pub type ConnectionIterator<'b> = FilterMap<
    Enumerate<std::slice::Iter<'b, ConnectionType>>,
    for<'a> fn((usize, &'a ConnectionType)) -> std::option::Option<Direction>,
>;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Connections {
    connections: [ConnectionType; 4],
}

pub const ALL_DIRECTIONS: Connections = Connections {
    connections: [ConnectionType::Road; 4],
};

pub const NO_DIRECTIONS: Connections = Connections {
    connections: [ConnectionType::None; 4],
};

impl Connections {
    pub fn new() -> Connections {
        Connections {
            ..Default::default()
        }
    }

    pub fn add(&mut self, conn: ConnectionType, dir: Direction) {
        self.connections[dir as usize] = conn;
    }

    pub fn remove(&mut self, dir: Direction) {
        self.connections[dir as usize] = ConnectionType::None;
    }

    pub fn count(&self) -> u32 {
        self.connections.iter().map(|conn| (*conn as u32).min(1)).sum()
    }

    pub fn filter_has_connection(my_arg: (usize, &ConnectionType)) -> Option<Direction> {
        if *my_arg.1 != ConnectionType::None {
            my_arg.0.try_into().ok()
        } else {
            None
        }
    }

    pub fn filter_has_no_connection(my_arg: (usize, &ConnectionType)) -> Option<Direction> {
        if *my_arg.1 == ConnectionType::None {
            my_arg.0.try_into().ok()
        } else {
            None
        }
    }

    pub fn iter(&self) -> ConnectionIterator {
        self.connections
            .iter()
            .enumerate()
            .filter_map(Connections::filter_has_connection)
    }

    pub fn iter_inverse(&self) -> ConnectionIterator {
        self.connections
            .iter()
            .enumerate()
            .filter_map(Connections::filter_has_no_connection)
    }

    pub fn has(&self, dir: Direction) -> bool {
        self.connections[dir as usize] != ConnectionType::None
    }

    pub fn has_layer(&self, dir: Direction, layer: ConnectionType) -> bool {
        self.connections[dir as usize] == layer
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
        } else if self.has_layer(Direction::Right, ConnectionType::Up) {
            write!(f, "u")
        } else if self.has_layer(Direction::Right, ConnectionType::Down) {
            write!(f, "d")
        } else {
            write!(f, "?")
        }
    }
}

#[cfg(test)]
mod connections_tests {
    // use std::mem;

    use super::*;

    #[test]
    fn test_new() {
        let connections = Connections::new();
        assert_eq!(connections.count(), 0, "Connections: {:?}", connections);

        // assert_eq!(mem::size_of::<ConnectionTest>(), 4);
    }

    #[test]
    fn test_iter() {
        let mut connection = Connections::new();
        connection.add(ConnectionType::Road, Direction::Right);
        connection.add(ConnectionType::Up, Direction::Left);
        assert_eq!(
            connection.iter().collect::<Vec<Direction>>(),
            vec![Direction::Left, Direction::Right]
        );

        assert_eq!(
            connection.iter_inverse().collect::<Vec<Direction>>(),
            vec![Direction::Up, Direction::Down]
        );
    }

    #[test]
    fn test_layer() {
        let mut connection = Connections::new();
        connection.add(ConnectionType::Up, Direction::Right);
        connection.add(ConnectionType::Road, Direction::Left);
        assert!(
            connection.iter().collect::<Vec<Direction>>()
                == vec![Direction::Left, Direction::Right]
        );
        // assert!(
        //     connection
        //         .iter_layer(ConnectionType::Up)
        //         .collect::<Vec<Direction>>()
        //         == vec![Direction::Right]
        // );

        assert_eq!(connection.count(), 2);
        // assert!(connection.safe_to_block() == true);
    }
}
