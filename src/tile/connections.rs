use std::
    fmt
;

use serde::{Deserialize, Serialize};

use crate::grid::Direction;

#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Connections {
    connections: Vec<Direction>,
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            ..Default::default()
        }
    }

    pub fn add(&mut self, dir: Direction) {
        if !self.connections.contains(&dir) {
            self.connections.push(dir);
        }
    }

    pub fn remove(&mut self, dir: Direction) {
        if let Some(index) = self.connections.iter_mut().position(|x| x == &dir) {
            self.connections.swap_remove(index);
        }
        // self.connections[dir as usize] = ConnectionType::None;
    }

    pub fn count(&self) -> u32 {
        self.connections.len() as u32
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Direction>  {
        self.connections.iter()
    }

    pub fn has(&self, dir: Direction) -> bool {
        self.connections.contains(&dir)
    }
}

impl fmt::Debug for Connections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has(Direction::UP) && self.has(Direction::LEFT) {
            write!(f, "r")
        } else if self.has(Direction::DOWN) && self.has(Direction::LEFT) {
            write!(f, "l")
        } else if self.has(Direction::DOWN) && self.has(Direction::RIGHT) {
            write!(f, "L")
        } else if self.has(Direction::UP) && self.has(Direction::RIGHT) {
            write!(f, "R")
        } else if self.has(Direction::LEFT) {
            write!(f, "<")
        } else if self.has(Direction::RIGHT) {
            write!(f, ">")
        } else if self.has(Direction::UP) {
            write!(f, "^")
        } else if self.has(Direction::DOWN) {
            write!(f, ".")
        // } else if self.has_layer(Direction::RIGHT, ConnectionType::Up) {
        //     write!(f, "u")
        // } else if self.has_layer(Direction::RIGHT, ConnectionType::Down) {
        //     write!(f, "d")
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
        connection.add(Direction::RIGHT);
        connection.add(Direction::LEFT);
        assert_eq!(
            connection.iter().collect::<Vec<&Direction>>(),
            vec![&Direction::RIGHT, &Direction::LEFT]
        );

        // assert_eq!(
        //     connection.iter_inverse().collect::<Vec<Direction>>(),
        //     vec![Direction::UP, Direction::DOWN]
        // );
    }

    #[test]
    fn test_layer() {
        // let mut connection = Connections::new();
        // connection.add(Direction::RIGHT);
        // connection.add(Direction::LEFT);
        // assert!(
        //     connection.iter().collect::<Vec<Direction>>()
        //         == vec![Direction::LEFT, Direction::RIGHT]
        // );
        // assert!(
        //     connection
        //         .iter_layer(ConnectionType::Up)
        //         .collect::<Vec<Direction>>()
        //         == vec![Direction::RIGHT]
        // );

        // assert_eq!(connection.count(), 2);
        // assert!(connection.safe_to_block() == true);
    }
}
