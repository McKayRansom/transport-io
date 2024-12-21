use crate::tile::{ConnectionType, House, Road, Tile};

use super::{Direction, Grid, Position, Z_BRIDGE};

#[derive(Debug)]
pub enum BuildError {
    InvalidTile,
    OccupiedTile,
    // TODO: HouseBlocking, etc...
}

pub type BuildResult = Result<(), BuildError>;

impl Tile {
    pub fn clear(&mut self) {
        *self = Tile::Empty;
    }

    pub fn build(&mut self, tile: Tile) -> BuildResult {
        if *self == Tile::Empty {
            *self = tile;
            Ok(())
        } else {
            Err(BuildError::OccupiedTile)
        }
    }

    pub fn edit_road<F>(&mut self, func: F) -> BuildResult
    where
        F: FnOnce(&mut Road),
    {
        match self {
            Tile::Empty => {
                let mut road = Road::new();
                func(&mut road);
                *self = Tile::Road(road);
                Ok(())
            }
            Tile::Road(road) => {
                func(road);
                Ok(())
            }
            _ => Err(BuildError::OccupiedTile),
        }
    }
}

impl Grid {
    pub fn build_bridge(&mut self, start_pos: Position, end_pos: Position) -> BuildResult {
        let (iter, dir) = start_pos.iter_line_to(end_pos);
        for pos in iter {
            let (build_pos, build_layer) = if pos == start_pos {
                (pos, ConnectionType::Up)
            } else if pos == end_pos {
                (pos.clone_on_layer(Z_BRIDGE), ConnectionType::Down)
            } else {
                (pos.clone_on_layer(Z_BRIDGE), ConnectionType::Road)
            };

            self.get_tile_mut(&build_pos)
                .ok_or(BuildError::InvalidTile)?
                .edit_road(|road| road.connect_layer(dir, build_layer))?;
        }

        Ok(())
    }

    pub fn build_road(&mut self, pos: &Position, dir: Direction) -> BuildResult {
        self.get_tile_mut(pos)
            .ok_or(BuildError::InvalidTile)?
            .edit_road(|road| road.connect(dir))
    }

    pub fn remove_road(&mut self, pos: &Position, dir: Direction) -> BuildResult {
        let tile = self.get_tile_mut(pos).ok_or(BuildError::InvalidTile)?;
        let mut remove_road = false;
        tile.edit_road(|road| {
            road.disconnect(dir);
            if road.connection_count() == 0 {
                remove_road = true;
            }
        })?;
        if remove_road {
            tile.clear();
        }

        Ok(())
    }

    pub fn clear(&mut self, pos: &Position) -> BuildResult {
        self.get_tile_mut(pos)
            .ok_or(BuildError::InvalidTile)?
            .clear();

        Ok(())
    }

    pub fn build_house(&mut self, pos: &Position) -> BuildResult {
        self.get_tile_mut(&pos)
            .ok_or(BuildError::InvalidTile)?
            .build(Tile::House(House::new()))
    }

    pub fn build_two_way_road(&mut self, pos: Position, dir: Direction) -> BuildResult {
        let pos = pos.round_to(2);

        let blueprint = if dir.is_horizontal() {
            Grid::new_from_string("<<\n>>")
        } else {
            Grid::new_from_string(".^\n.^")
        };

        for (y, row) in blueprint.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                self.build_road(
                    &self.pos(x as i16 + pos.x, y as i16 + pos.y),
                    tile.ground.iter_connections().next().unwrap(),
                )?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod grid_build_tests {
    use crate::{grid::Direction, tile::Tile};

    use super::*;

    #[test]
    fn test_build() -> BuildResult {
        let mut grid = Grid::new_from_string("___");

        let pos = grid.pos(0, 0);

        grid.build_road(&pos, Direction::Right)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        grid.clear(&pos)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('e'));

        grid.build_road(&pos, Direction::Right)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        grid.build_road(&pos, Direction::Up)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('R'));

        grid.remove_road(&pos, Direction::Up)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        grid.remove_road(&pos, Direction::Right)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('e'));

        Ok(())
    }

    #[test]
    fn test_build_bridge() -> BuildResult {
        let mut grid = Grid::new_from_string("____");

        grid.build_bridge(grid.pos(0, 0), grid.pos(3, 0))?;

        // TODO: Fix this??
        // assert_eq!(grid, Grid::new_from_string("ueee"));

        Ok(())
    }

    #[test]
    fn test_build_two_way_road() -> BuildResult {
        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::Left)?;
        assert_eq!(grid, Grid::new_from_string("<<__\n>>__"));

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::Down)?;
        assert_eq!(grid, Grid::new_from_string(".^__\n.^__"));

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::Right)?;
        grid.build_two_way_road(grid.pos(0, 0), Direction::Up)?;
        assert_eq!(grid, Grid::new_from_string("lr__\nLR__"));

        Ok(())
    }
}
