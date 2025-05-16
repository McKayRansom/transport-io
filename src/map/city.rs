use macroquad::prelude::rand;
use pathfinding::prelude::bfs_reach;
use serde::{Deserialize, Serialize};

use super::{
    building::Building,
    grid::Grid,
    tile::{Road, Tile},
    Direction, Position,
};

use crate::hash_map_id::Id;

const CITY_GROW_TICKS: u32 = 16 * 10;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct City {
    pub pos: Position,
    pub name: String,
    // pub rating: f32,
    grow_ticks: u32,
    pub grow_rate: u32,
    id: Id,
    pub houses: Vec<Id>,
    // vehicle_on_the_way: Option<Id>,
    pub houses_goal_met: usize,
}

impl City {
    pub fn new(id: Id, pos: Position, name: String) -> Self {
        City {
            pos,
            name,
            grow_ticks: rand::gen_range(0, CITY_GROW_TICKS),
            grow_rate: CITY_GROW_TICKS,
            id,
            houses: Vec::new(),
            houses_goal_met: 0,
        }
    }

    /*
     * TODO: Make a more interesting growth algo:
     * https://github.com/OpenTTD/OpenTTD/blob/521d0465d9ad1be276668c58b2f004f8d17ff103/src/town_cmd.cpp#L1554
     * Classic OpenTTD Algo (As I understand the complicated cpp code):
     * - Find a tile near the city center that has a road
     * - Iterate reachable road tiles (essentially BFS but they don't call it that)
     * - For classic non-grid rules appear to be:
     *   - If tile is a parital road (dead end road maybe?) grow the road
     *   - If it's empty we can build a road only if there's at least one house next to us and we don't create a curved road?
     *
     */
    pub fn grow(&mut self, grid: &mut Grid) {
        // for now let's just assume the city position is a road
        let start = self.pos;
        let tile = grid.get_tile(&start).unwrap();

        // TEMP: In future a better solution?
        if !matches!(tile, Tile::Road(_)) {
            *grid.get_tile_mut(&start).unwrap() = Tile::Road(Road::new_connected(&Direction::ALL));
        }

        for (pos, dir_from) in bfs_reach((start, Direction::NONE), |(pos, _dir)| {
            if grid.get_tile(pos).is_some_and(Tile::is_road) {
                // NOTE: RANDOM might be better here!
                Direction::ALL
                    .iter()
                    .map(|dir| (*pos + *dir, *dir))
                    .collect()
            } else {
                Vec::new()
            }
        }) {
            let Some(tile) = grid.get_tile(&pos) else {
                continue;
            };

            if !tile.is_empty() {
                continue;
            }

            let mut road_dir: Option<Direction> = None;
            // GRID_LAYOUT:
            // let diff = pos - start;
            // const GRID_SIZE: i8 = 3;
            // let should_build_road = diff.x % GRID_SIZE == 0 || diff.y % GRID_SIZE == 0;
            // RANDOM_LAYOUT
            let prev_tile = grid.get_tile(&(pos + dir_from.inverse())).unwrap();
            let left_tile = grid.get_tile(&(pos + dir_from.rotate_left()));
            let right_tile = grid.get_tile(&(pos + dir_from.rotate_right()));

            // don't create double intersections if possible
            let left_and_right_not_roads = left_tile.is_some_and(|tile| !tile.is_road())
                && right_tile.is_some_and(|tile| !tile.is_road());

            let should_build_road = left_and_right_not_roads
                // continue existing roads when possible
                && (if let Tile::Road(road) = prev_tile {
                    // (this should always be true...)
                    road_dir = Some(dir_from);
                    road.connections.is_connected(dir_from)
                } else {
                    false
                } 
                // some chance of starting a new road
                || rand::rand() % 4 == 0);
            // if found_a_house && !should_build_road {
            //     should_build_road = rand::rand() % 4 == 0;
            // }

            if should_build_road {
                // build a road
                let mut dirs: Vec<Direction> = Direction::ALL
                    .into_iter()
                    .filter(|dir| {
                        // always connect to adjacent roads (for now maybe?)
                        let Some(Tile::Road(road)) = grid.get_tile_mut(&(pos + *dir)) else {
                            return false;
                        };
                        road.connections.connect(dir.inverse());

                        true
                    })
                    .collect();

                // dirs.push(Direction::ALL[rand::rand() as usize % Direction::ALL.len()]);
                // always continue existing roads
                if let Some(dir) = road_dir {
                    dirs.push(dir);
                }

                *grid.get_tile_mut(&pos).unwrap() = Tile::Road(Road::new_connected(&dirs));
                return;
            } else {
                // build a house
                let building_id = grid.buildings.insert(Building::new_house(pos, self.id));
                *grid.get_tile_mut(&pos).unwrap() = Tile::Building(building_id);
                self.houses.push(building_id);
                return;
            }
        }

        log::info!("Failed to grow city ")

        // let start_house_id = self.random_house();
        // if let Some(building) = grid.buildings.hash_map.get(&start_house_id) {
        //     let mut building_pos = building.pos;
        //     let dir = Direction::random();
        //     loop {
        //         let pos = building_pos + dir;
        //         building_pos = pos;
        //         match grid.is_pos_clear(&pos) {
        //             Ok(_) => return Some(Building::new_house(pos, self.id)),
        //             Err(BuildError::OccupiedTile) => continue,
        //             Err(BuildError::InvalidTile) => break,
        //             Err(BuildError::InsufficientFunds) => unreachable!(),
        //         }
        //     }
        // } else {
        //     // TODO: Remove the house from our list
        // }
    }

    // This returns a building for historic reasons but I think now that buildings are stored on the grid we can fix this...
    pub fn update(&mut self, grid: &mut Grid) {
        // Only grow if all houses have their "goal met"
        self.houses_goal_met = 0;
        for (i, building_id) in self.houses.iter().enumerate() {
            if let Some(buliding) = grid.buildings.hash_map.get(building_id) {
                if buliding.arrived_count > 0 {
                    self.houses_goal_met += 1;
                }
            } else {
                // old building :(
                self.houses.swap_remove(i);
                return;
            }
        }
        if self.houses.len() != self.houses_goal_met {
            return;
        }
        self.grow_ticks += 1;
        if self.grow_ticks > self.grow_rate {
            self.grow_ticks = 0;
            self.grow(grid);
        }
    }

    pub fn random_house(&self) -> Id {
        let index = rand::gen_range(0, self.houses.len());
        self.houses[index]
    }
}

#[cfg(test)]
mod city_tests {
    use super::*;

    #[test_log::test]
    fn test_city_growth() {
        let mut grid = Grid::new((32, 32));
        let mut city = City::new(1234, Position::new(16, 16), "test".into());

        for _ in 0..128 {
            city.grow(&mut grid);
        }
        dbg!(grid);

        // assert!(false);
    }
}
