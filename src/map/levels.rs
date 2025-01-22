use super::{Map, Unlocked, DEFAULT_CITY_ID};

// pub const TEST_LEVEL: Option<usize> = Some(1);
pub const TEST_LEVEL: Option<usize> = None;

pub struct Level {
    pub name: &'static str,
    pub hint: &'static str,
    pub unlocked: Unlocked,
    pub grid: &'static str,
}

const LEVELS: &[Level] = &[
    Level {
        name: "Connect Road",
        hint: "Select Build Road, then click and drag to connect the colors",
        unlocked: Unlocked::Roads,
        grid: "
            __________________
            __________________
            __________________
            __________________
            11______________22
            11______________22
            __________________
            __________________
            __________________
            __________________
            ",
    },
    Level {
        name: "Intersection",
        hint: "Intersections are created when roads cross",
        unlocked: Unlocked::Roads,
        grid: "
            ________22________
            ________22________
            __________________
            __________________
            __________________
            __________________
            __________________
            __________________
            11______________33
            11______________33
            __________________
            __________________
            __________________
            __________________
            ",
    },
    Level {
        name: "Bridges",
        hint: "Bridges can be build over Rivers or roads",
        unlocked: Unlocked::Bridges,
        grid: "
            ________ww________
            ________ww________
            ________ww________
            ________ww________
            11______ww______22
            11______ww______22
            ________ww________
            ________ww________
            ________ww________
            ________ww________
            ",
    },
    Level {
        name: "Highway",
        hint: "You can create one-way roads",
        unlocked: Unlocked::OneWayRoads,
        grid: "
            __________________
            __________________
            11______________22
            11______________22
            11______________22
            11______________22
            __________________
            __________________
            ",
    },
];

pub const LEVEL_COUNT: usize = LEVELS.len();

pub fn new_level(level_number: usize) -> Map {
    let level: &Level = &LEVELS[level_number];

    let mut map = Map::new_from_string(level.grid);

    map.metadata.is_level = true;
    map.metadata.grow_cities = false;
    map.metadata.level_complete = false;
    map.metadata.level_number = level_number;
    map.metadata.unlocks = level.unlocked;
    map.metadata.level_hint = Some(level.hint.into());

    map.get_city_mut(DEFAULT_CITY_ID).unwrap().name = level.name.into();

    map
}

#[cfg(test)]
mod level_tests {
    use super::*;

    #[test]
    fn test_new_without_panic() {
        for i in 0..LEVEL_COUNT {
            new_level(i);
        }
    }
}
