use std::ops::{Add, Sub};

use macroquad::math::Rect;
use serde::{Deserialize, Serialize};

use crate::{
    hash_map_id::Id,
    map::{position::GRID_CELL_SIZE, Direction, Position},
};

use super::{Reservation, ReservationError, ReservedList, Tick};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct RoadPosition {
    pub pos: Position,
    pub off: (i8, i8),
}

impl RoadPosition {
    pub fn new(pos: (i16, i16), off: (i8, i8)) -> Self {
        Self {
            pos: pos.into(),
            off,
        }
    }

    pub fn index(&self) -> usize {
        (self.off.0 + self.off.1 * 2) as usize
    }

    pub fn corner_pos(pos: Position, dir: Direction) -> Self {
        match dir {
            Direction::LEFT => Self { pos, off: (1, 1) },
            Direction::RIGHT => Self { pos, off: (0, 0) },
            Direction::DOWN => Self { pos, off: (1, 0) },
            Direction::UP => Self { pos, off: (0, 1) },
            // Direction::NONE => pos,
            _ => Self { pos, off: (0, 0) },
        }
    }

    const TOP_LEFT_DIRS: [Direction; 2] = [Direction::DOWN, Direction::LEFT];
    const TOP_RIGHT_DIRS: [Direction; 2] = [Direction::LEFT, Direction::UP];
    const BOT_RIGHT_DIRS: [Direction; 2] = [Direction::UP, Direction::RIGHT];
    const BOT_LEFT_DIRS: [Direction; 2] = [Direction::RIGHT, Direction::DOWN];

    // .<
    // >^
    pub fn default_connections(&self) -> &'static [Direction] {
        match self.off {
            (0, 0) => &Self::TOP_LEFT_DIRS,
            (1, 0) => &Self::TOP_RIGHT_DIRS,
            (1, 1) => &Self::BOT_RIGHT_DIRS,
            (0, 1) => &Self::BOT_LEFT_DIRS,
            _ => &[],
        }
    }

    pub fn successor(&self, dir: Direction) -> RoadPosition {
        if dir.f || dir.flatten() == self.default_connections()[1] {
            // we can go directly to the next tile so do that
            *self + dir
        } else {
            // start turning around...
            *self + self.default_connections()[0]

            // TODO: If the conn is one-way we can just go that way always?
            // BUG: This doesn't handle LAYER_UP properly...
            // *self + dir
        }
    }

    pub fn iter_succ(&self, next_pos: Position, one_way: bool) -> RoadPositionSuccIter {
        RoadPositionSuccIter {
            pos: *self,
            next_pos,
            one_way,
        }
    }

    pub fn one_way_alt(&self, dir: Direction) -> RoadPosition {

        // TODO: THIS IS DEF WRONG, BUT I LIKE THAT IT POPS ENTERING VEHICLES OVER
        let mut new_pos = *self;
        if dir.x == 0 {
            new_pos.off.1 ^= 1;
        } else if dir.y == 0 {
            new_pos.off.0 ^= 1;
        }

        new_pos
    } 
}

pub struct RoadPositionSuccIter {
    pos: RoadPosition,
    next_pos: Position,
    one_way: bool,
}

impl Iterator for RoadPositionSuccIter {
    type Item = RoadPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.pos == self.next_pos {
            None
        } else {
            let mut dir = self.next_pos - self.pos.pos;
            if self.one_way {
                dir.f = true;
            }
            self.pos = self.pos.successor(dir);
            Some(self.pos)
        }
    }
}

impl From<(i16, i16)> for RoadPosition {
    fn from(value: (i16, i16)) -> Self {
        Self {
            pos: value.into(),
            off: (0, 0),
        }
    }
}

impl From<((i16, i16), (i8, i8))> for RoadPosition {
    fn from(value: ((i16, i16), (i8, i8))) -> Self {
        Self {
            pos: value.0.into(),
            off: value.1,
        }
    }
}

impl From<Position> for RoadPosition {
    fn from(pos: Position) -> Self {
        Self { pos, off: (0, 0) }
    }
}

impl From<RoadPosition> for Position {
    fn from(pos: RoadPosition) -> Self {
        pos.pos
    }
}

impl From<RoadPosition> for Rect {
    fn from(pos: RoadPosition) -> Self {
        Rect::new(
            pos.pos.x as f32 * GRID_CELL_SIZE.0 + pos.off.0 as f32 * GRID_CELL_SIZE.0 / 2.,
            pos.pos.y as f32 * GRID_CELL_SIZE.1 + pos.off.1 as f32 * GRID_CELL_SIZE.1 / 2.,
            GRID_CELL_SIZE.0 / 2.,
            GRID_CELL_SIZE.1 / 2.,
        )
    }
}

impl Sub<RoadPosition> for RoadPosition {
    type Output = Direction;
    fn sub(self, rhs: RoadPosition) -> Self::Output {
        let dir = self.pos - rhs.pos;
        if dir != Direction::NONE {
            dir
        } else {
            Direction::new(self.off.0 - rhs.off.0, self.off.1 - rhs.off.1, 0)
        }
    }
}

impl Add<Direction> for RoadPosition {
    type Output = RoadPosition;

    fn add(self, rhs: Direction) -> Self::Output {
        let mut new_pos = Self {
            pos: self.pos,
            off: self.off,
        };

        new_pos.pos.z += rhs.z as i16;

        new_pos.off.0 += rhs.x;
        new_pos.off.1 += rhs.y;

        // There is probably a better way to do this...
        if new_pos.off.0 < 0 {
            new_pos.pos.x -= 1;
            new_pos.off.0 = 1;
        } else if new_pos.off.0 > 1 {
            new_pos.pos.x += 1;
            new_pos.off.0 = 0;
        }

        // There is probably a better way to do this...
        if new_pos.off.1 < 0 {
            new_pos.pos.y -= 1;
            new_pos.off.1 = 1;
        } else if new_pos.off.1 > 1 {
            new_pos.pos.y += 1;
            new_pos.off.1 = 0;
        }

        new_pos
    }
}

#[cfg(test)]
mod road_pos_tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(
            RoadPosition::new((0, 0), (0, 0)) + Direction::RIGHT,
            RoadPosition::new((0, 0), (1, 0))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 0)) + Direction::RIGHT,
            RoadPosition::new((1, 0), (0, 0))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 1)) + Direction::RIGHT,
            RoadPosition::new((1, 0), (0, 1))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 1)) + Direction::DOWN_RIGHT,
            RoadPosition::new((1, 1), (0, 0))
        );
        // assert_eq!(
        //     RoadPosition::new((1, 1), (0, 0)) + Direction::UP_LEFT,
        //     RoadPosition::new((0, 0), (1, 1))
        // );
    }

    #[test]
    fn test_round_dir() {
        assert_eq!(
            RoadPosition::new((0, 0), (0, 0)).successor(Direction::DOWN),
            RoadPosition::new((0, 0), (0, 1))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 1)).successor(Direction::RIGHT),
            RoadPosition::new((1, 0), (0, 1))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 1)).successor(Direction::RIGHT),
            RoadPosition::new((1, 0), (0, 1))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 1)).successor(Direction::RIGHT_UP),
            RoadPosition {
                pos: Position::_new_z(1, 0, 1),
                off: (0, 1)
            }
        );
    }

    #[test]
    fn test_one_way() {
        assert_eq!(
            RoadPosition::new((0, 0), (1, 0)).successor(Direction::RIGHT),
            RoadPosition::new((0, 0), (0, 0))
        );

        assert_eq!(
            RoadPosition::new((0, 0), (1, 0)).successor(Direction::RIGHT_F),
            RoadPosition::new((1, 0), (0, 0))
        );
    }

    #[test]
    fn test_pos_alt() {
        assert_eq!(
            RoadPosition::new((0, 0), (1, 0)).one_way_alt(Direction::RIGHT),
            RoadPosition::new((0, 0), (0, 0))
        );
        assert_eq!(
            RoadPosition::new((0, 0), (1, 0)).one_way_alt(Direction::UP),
            RoadPosition::new((0, 0), (1, 1))
        );
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connections {
    cons: [u8; 4],
}

impl Connections {
    // This could be a bitfield
    const CONNECTED: u8 = 0b1;
    const LAYER_UP: u8 = 0b10;
    const LAYER_DOWN: u8 = 0b100;
    const ONE_WAY: u8 = 0b1000;

    fn index(dir: Direction) -> usize {
        match dir.flatten() {
            Direction::RIGHT => 0,
            Direction::DOWN => 1,
            Direction::LEFT => 2,
            Direction::UP => 3,
            _ => 4,
        }
    }

    fn dir(index: usize, con: u8) -> Option<Direction> {
        if con & Self::CONNECTED == 0 {
            return None;
        }
        let mut dir = match index {
            0 => Direction::RIGHT,
            1 => Direction::DOWN,
            2 => Direction::LEFT,
            3 => Direction::UP,
            _ => return None,
        };
        if con & Self::LAYER_UP == Self::LAYER_UP {
            dir.z = 1;
        } else if con & Self::LAYER_DOWN == Self::LAYER_DOWN {
            dir.z = -1;
        }
        dir.f = con & Self::ONE_WAY == Self::ONE_WAY;

        Some(dir)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        self.cons.get_mut(index)
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.cons.get(index)
    }

    pub fn connect(&mut self, dir: Direction) {
        let mut connect: u8 = Self::CONNECTED;
        if dir.f {
            connect |= Self::ONE_WAY;
        }
        match dir.z {
            -1 => connect |= Self::LAYER_DOWN,
            1 => connect |= Self::LAYER_UP,
            _ => {}
        }

        if let Some(con) = self.get_mut(Self::index(dir)) {
            *con = connect;
        }
    }

    pub fn disconnect(&mut self, dir: Direction) {
        if let Some(con) = self.get_mut(Self::index(dir)) {
            *con = 0;
        }
    }

    pub fn count(&self) -> usize {
        self.cons
            .iter()
            .fold(0, |sum, con| sum + (con & Self::CONNECTED) as usize)
    }

    pub fn is_one_way(&self, dir: Direction) -> bool {
        let index = Self::index(dir);
        self.get(index)
            .is_some_and(|c| *c & Self::ONE_WAY == Self::ONE_WAY)
    }

    pub fn is_connected(&self, dir: Direction) -> bool {
        let index = Self::index(dir);
        self.get(index)
            .is_some_and(|c| Self::dir(index, *c) == Some(dir))
    }

    pub fn is_connected_flat(&self, dir: Direction) -> bool {
        let index = Self::index(dir);
        self.get(index)
            .is_some_and(|c| *c & Self::CONNECTED == Self::CONNECTED)
    }

    pub fn get_nth_dir(&self, mut n: usize) -> Option<Direction> {
        for (index, con) in self.cons.iter().enumerate() {
            if *con & Self::CONNECTED == Self::CONNECTED {
                if n == 0 {
                    return Self::dir(index, *con);
                }
                n -= 1;
            }
        }
        None
    }

    pub fn iter(&self) -> ConnectionsIter {
        ConnectionsIter {
            cons: self.clone(),
            i: 0,
        }
    }
}

pub struct ConnectionsIter {
    // TODO: THIS SUCKS LOL
    cons: Connections,
    i: usize,
}

impl Iterator for ConnectionsIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        let dir = self.cons.get_nth_dir(self.i)?;
        self.i += 1;
        Some(dir)
    }
}

impl From<&[Direction]> for Connections {
    fn from(value: &[Direction]) -> Self {
        let mut connection = Connections::default();
        for dir in value {
            connection.connect(*dir);
        }
        connection
    }
}

#[cfg(test)]
mod road_connections_tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut con = Connections::default();
        con.connect(Direction::DOWN);
        assert!(con.is_connected(Direction::DOWN));
        assert_eq!(con.count(), 1);
        con.disconnect(Direction::DOWN);
        assert!(!con.is_connected(Direction::DOWN));
        assert_eq!(con.count(), 0);
    }

    #[test]
    fn test_iter() {
        let mut con = Connections::default();
        for dir in Direction::ALL {
            con.connect(dir);
        }

        assert_eq!(
            con.iter().collect::<Vec<Direction>>(),
            Direction::ALL.to_vec()
        )
    }

    #[test]
    fn test_layers() {
        let mut con = Connections::default();
        con.connect(Direction::LEFT + Direction::LAYER_UP);
        assert!(con.is_connected(Direction::LEFT + Direction::LAYER_UP));
        // assert!(!con.is_connected(Direction::LEFT));
        assert_eq!(
            con.get_nth_dir(0).unwrap(),
            Direction::LEFT + Direction::LAYER_UP
        );

        assert_eq!(
            con.iter().next().unwrap(),
            Direction::LEFT + Direction::LAYER_UP
        )
    }
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Road {
    pub station: Option<Id>,
    pub reservations: [ReservedList; 4],
    pub connections: Connections,
}

impl Road {
    pub fn new_connected(dirs: &[Direction]) -> Self {
        Self {
            connections: dirs.into(),
            ..Default::default()
        }
    }

    pub fn new_station(station: Option<Id>) -> Self {
        Self {
            station,
            ..Default::default()
        }
    }

    pub fn new() -> Self {
        Road {
            ..Default::default()
        }
    }

    pub fn unreserve(&mut self, id: Id, road_pos: RoadPosition) {
        self.reservations[road_pos.index()].unreserve(id);
    }

    pub fn try_reserve(
        &mut self,
        id: Id,
        current: Tick,
        reservation: &Reservation,
    ) -> Result<(), ReservationError> {
        self.reservations[reservation.pos.index()].try_reserve(id, current, reservation)
    }

    pub fn is_reserved(&self, id: Id, pos: RoadPosition, start: Tick, end: Tick) -> bool {
        self.reservations[pos.index()].is_reserved(id, start, end)
    }
}

const ROAD_DBG_CHAR: &[(char, &[Direction])] = &[
    ('>', &[Direction::RIGHT]),
    ('<', &[Direction::LEFT]),
    ('^', &[Direction::UP]),
    ('.', &[Direction::DOWN]),
    ('d', &[Direction::RIGHT_F]),
    ('a', &[Direction::LEFT_F]),
    ('w', &[Direction::UP_F]),
    ('s', &[Direction::DOWN_F]),
    // Roundabouts - top left
    ('l', &[Direction::LEFT, Direction::DOWN]),
    // Roundabouts - top right
    ('r', &[Direction::LEFT, Direction::UP]),
    // Roundabouts - bottom Left
    ('L', &[Direction::RIGHT, Direction::DOWN]),
    // Roundabouts - bottom Right
    ('R', &[Direction::RIGHT, Direction::UP]),
    ('|', &[Direction::DOWN, Direction::UP]),
    ('-', &[Direction::RIGHT, Direction::LEFT]),
    ('+', &Direction::ALL),
    ('}', &[Direction::RIGHT_UP]),
    ('{', &[Direction::LEFT_UP]),
    (']', &[Direction::RIGHT_DOWN, Direction::LEFT]),
    ('[', &[Direction::LEFT_DOWN, Direction::RIGHT]),
    ('*', &[Direction::NONE]), // unconnected road
];

impl TryFrom<char> for Road {
    type Error = ();
    fn try_from(c: char) -> Result<Road, ()> {
        let mut road = Road::new();
        if let Some(elem) = ROAD_DBG_CHAR.iter().find(|elem| elem.0 == c) {
            for dir in elem.1 {
                road.connections.connect(*dir);
            }
            Ok(road)
        } else {
            Err(())
        }
    }
}

impl std::fmt::Debug for Road {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(elem) = ROAD_DBG_CHAR.iter().find(|elem| {
            let dirs = elem.1;
            if dirs.len() != self.connections.count() {
                return false;
            }
            for dir in dirs {
                if !self.connections.is_connected(*dir) {
                    return false;
                }
            }
            return true;
        }) {
            write!(f, "{}", elem.0)
        } else {
            write!(f, "*")
        }
    }
}

#[cfg(test)]
mod road_tests {
    // use std::mem;

    use super::*;

    #[test]
    fn test_new() {
        let _road = Road::new();
        // assert_eq!(road.connectconnection_count(), 0, "Connections: {:?}", road);
    }
}
