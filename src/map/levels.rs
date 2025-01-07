use super::{city::City, Map, Position};

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

        let city_id = map.cities.insert(City::new(
            map.cities.id,
            (LEVEL_MAP_SIZE.0 / 2, LEVEL_MAP_SIZE.1 / 2).into(),
            format!("level {level}").into(),
        ));

        map.cities.hash_map.get_mut(&city_id).unwrap().grow_rate = std::u32::MAX;

        match level {
            0 => {
                map.new_spawner(LEFT_POS);
                map.new_spawner(RIGHT_POS);
            }
            1 => {
                map.new_spawner(LEFT_POS);
                map.new_spawner(RIGHT_POS);
                map.new_spawner(TOP_POS);
                map.new_spawner(BOT_POS);
            }
            _ => {}
        }

        map
    }

    pub fn new_spawner(&mut self, pos: Position) {
        self.cities
            .hash_map
            .get_mut(&1)
            .unwrap()
            .generate_building(pos, &mut self.buildings, &mut self.grid)
            .unwrap();
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
