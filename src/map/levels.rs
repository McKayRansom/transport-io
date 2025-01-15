use crate::consts::SpawnerColors;

use super::{
    build::{BuildAction, BuildActionStation}, building::Building, Direction, Map, Position
};

const LEVEL_MAP_SIZE: (i16, i16) = (2 * 9, 2 * 9);

pub const LEVEL_COUNT: usize = 10;
// pub const TEST_LEVEL: Option<usize> = Some(1);
pub const TEST_LEVEL: Option<usize> = None;

pub const LEFT_POS: Position = Position::new(0, LEVEL_MAP_SIZE.1 / 2);
pub const RIGHT_POS: Position = Position::new(LEVEL_MAP_SIZE.0 - 2, LEVEL_MAP_SIZE.1 / 2);
pub const TOP_POS: Position = Position::new(LEVEL_MAP_SIZE.0 / 2, 0);
pub const BOT_POS: Position = Position::new(LEVEL_MAP_SIZE.0 / 2, LEVEL_MAP_SIZE.1 - 2);

impl Map {
    pub fn new_level(level: usize) -> Self {
        let mut map = Self::new_blank(LEVEL_MAP_SIZE);

        map.metadata.is_level = true;
        map.metadata.level_complete = false;
        map.metadata.level_number = level;

        let city_id = map.new_city(
            (LEVEL_MAP_SIZE.0 / 2, LEVEL_MAP_SIZE.1 / 2).into(),
            format!("level {level}"),
        );

        // WOOF
        map.get_city_mut(city_id).unwrap().grow_rate = u32::MAX;

        match level {
            0 => {
                map.new_spawner(LEFT_POS, Direction::RIGHT, SpawnerColors::Blue);
                map.new_spawner(RIGHT_POS, Direction::LEFT, SpawnerColors::Red);
            }
            1 => {
                map.new_spawner(LEFT_POS, Direction::RIGHT, SpawnerColors::Blue);
                map.new_spawner(RIGHT_POS, Direction::LEFT, SpawnerColors::Red);
                map.new_spawner(TOP_POS, Direction::DOWN, SpawnerColors::Green);
                map.new_spawner(BOT_POS, Direction::UP, SpawnerColors::Yellow);
            }
            _ => {}
        }

        map
    }

    pub fn new_spawner(&mut self, pos: Position, dir: Direction, color: SpawnerColors) {
        let pos = pos.round_to(2);
        BuildActionStation::new(self, pos, Building::new_spawner(pos, dir, color, 1))
            .execute(self)
            .unwrap()
    }
}

#[cfg(test)]
mod level_tests {
    use super::*;

    #[test]
    fn test_new_without_panic() {
        for i in 0..LEVEL_COUNT {
            Map::new_level(i);
        }
    }
}
