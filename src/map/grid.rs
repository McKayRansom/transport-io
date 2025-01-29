use serde::{Deserialize, Serialize};
use std::fmt;

use super::building::Building;
use super::position::GRID_CELL_SIZE;
use super::tile::Tile;
use super::{BuildingHashMap, Direction, Position, DEFAULT_CITY_ID};
use crate::consts::SpawnerColors;
use crate::hash_map_id::HashMapId;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GridTile {
    pub ground: Tile,
    pub bridge: Tile,
}

impl GridTile {
    fn new() -> Self {
        GridTile {
            ground: Tile::new(),
            bridge: Tile::new(),
        }
    }

    fn new_from_char(chr: char) -> Self {
        GridTile {
            ground: Tile::new_from_char(chr),
            bridge: Tile::Empty,
        }
    }

    fn new_from_char_layers(args: (char, char)) -> Self {
        GridTile {
            ground: Tile::new_from_char(args.0),
            bridge: Tile::new_from_char(args.1),
        }
    }

    fn get(&self, pos_z: i16) -> Option<&Tile> {
        if pos_z == 0 {
            Some(&self.ground)
        } else if pos_z == 1 {
            Some(&self.bridge)
        } else {
            None
        }
    }

    fn get_mut(&mut self, pos_z: i16) -> Option<&mut Tile> {
        if pos_z == 0 {
            Some(&mut self.ground)
        } else if pos_z == 1 {
            Some(&mut self.bridge)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct Grid {
    pub tiles: Vec<Vec<GridTile>>,
    pub buildings: BuildingHashMap,
}

impl Position {}

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for y in 0..self.tiles.len() {
            let mut has_bridges = false;
            for x in 0..self.tiles[y].len() {
                self.tiles[y][x].ground.fmt(f)?;
                has_bridges |= self.tiles[y][x].bridge != Tile::Empty;
            }
            if has_bridges {
                write!(f, "  ")?;
                for x in 0..self.tiles[y].len() {
                    self.tiles[y][x].bridge.fmt(f)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid {
    pub fn new(size: (i16, i16)) -> Self {
        Grid {
            tiles: vec![vec![GridTile::new(); size.0 as usize]; size.1 as usize],
            buildings: HashMapId::new(),
        }
    }

    pub fn size(&self) -> (i16, i16) {
        (self.tiles[0].len() as i16, self.tiles.len() as i16)
    }

    pub fn size_px(&self) -> (f32, f32) {
        (
            self.tiles[0].len() as f32 * GRID_CELL_SIZE.0,
            self.tiles.len() as f32 * GRID_CELL_SIZE.1,
        )
    }

    #[allow(dead_code)]
    pub fn pos(&self, x: i16, y: i16) -> Position {
        Position::new(x, y)
    }

    pub fn new_from_string(string: &str) -> Grid {
        let mut grid = Grid {
            buildings: HashMapId::new(),
            tiles: string
                .split_ascii_whitespace()
                .map(|line| line.chars().map(GridTile::new_from_char).collect())
                .collect(),
        };

        grid.fixup_from_string();

        grid
    }

    fn fixup_from_string(&mut self) {
        let size = self.size();
        for pos in Position::new(0, 0).iter_area(Direction::new(size.0 as i8, size.1 as i8, 0)) {
            match self.get_tile_mut(&pos).unwrap() {
                Tile::Building(_) => {
                    let building_id = self
                        .buildings
                        .insert(Building::new_house(pos, DEFAULT_CITY_ID));

                    *self.get_tile_mut(&pos).unwrap() = Tile::Building(building_id);
                }
                Tile::Road(road) => {
                    if let Some(station) = road.station {
                        self.buildings.hash_map.entry(station).or_insert_with(|| {
                            let dir = if pos.x < size.0 / 4 {
                                Direction::RIGHT
                            } else if pos.y < size.1 / 4 {
                                Direction::DOWN
                            } else if pos.x > (size.0 * 3) / 4 {
                                Direction::LEFT
                            } else {
                                Direction::UP
                            };
                            Building::new_spawner(
                                pos,
                                dir,
                                SpawnerColors::from_number(station),
                                DEFAULT_CITY_ID,
                            )
                        });
                    }
                }
                _ => {}
            }
        }
    }

    #[allow(dead_code)]
    pub fn new_from_string_layers(string: &str, bridge_layer: &str) -> Grid {
        let tiles: Vec<Vec<GridTile>> = string
            .split_ascii_whitespace()
            .zip(bridge_layer.split_ascii_whitespace())
            .map(|(line, bridge_line)| {
                line.chars()
                    .zip(bridge_line.chars())
                    .map(GridTile::new_from_char_layers)
                    .collect()
            })
            .collect();

        Grid {
            tiles,
            buildings: HashMapId::new(),
        }
    }

    pub fn get_tile(&self, pos: &Position) -> Option<&Tile> {
        self.tiles
            .get(pos.y as usize)?
            .get(pos.x as usize)?
            .get(pos.z)
    }

    pub fn get_tile_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        self.tiles
            .get_mut(pos.y as usize)?
            .get_mut(pos.x as usize)?
            .get_mut(pos.z)
    }
}

#[cfg(test)]
mod grid_tests {
    use crate::map::tile::Road;

    use super::*;

    #[test]
    fn test_new() {
        let grid = Grid::new((3, 1));
        assert_eq!(*grid.get_tile(&grid.pos(0, 0)).unwrap(), Tile::Empty);

        assert_eq!(grid.size(), (3, 1));

        assert!(grid.get_tile(&(3, 0).into()).is_none());
        assert!(grid.get_tile(&(0, 1).into()).is_none());
    }

    #[test]
    fn test_new_from_string() {
        let grid = Grid::new_from_string(
            "<<<
             >>>",
        );

        assert_eq!(format!("{:?}", grid), "\n<<<\n>>>\n");

        assert_eq!(
            *grid.get_tile(&grid.pos(1, 1)).unwrap(),
            Tile::Road(Road::new_connected(Direction::RIGHT, None))
        );
    }
}
