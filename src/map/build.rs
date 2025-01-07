use crate::{hash_map_id::{HashMapId, Id}, map::Map};

use super::{
    building::{Building, BUILDING_SIZE},
    grid::Grid,
    tile::{Ramp, Road, Tile},
    Direction, Position,
};

#[derive(Debug)]
pub enum BuildError {
    InvalidTile,
    OccupiedTile,
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
        let start_pos_up = start_pos + Direction::LAYER_UP;
        let end_pos_up = end_pos + Direction::LAYER_UP;
        let (iter, dir) = start_pos_up.iter_line_to(end_pos_up);
        for pos in iter {
            if pos == start_pos_up {
                self.build_road(pos, dir)?;
                self.build_ramp(&start_pos, Direction::LAYER_UP)?;
            } else if pos != end_pos_up {
                self.build_road(pos, dir)?;
            } else {
                self.build_road(pos, dir + Direction::LAYER_DOWN)?;
                self.build_ramp(&end_pos, Direction::NONE)?;
            };
        }

        Ok(())
    }

    fn build_ramp(&mut self, pos: &Position, dir: Direction) -> BuildResult {
        // let pos = &pos.round_to(2);
        self.get_tile_mut(pos)
            .ok_or(BuildError::InvalidTile)?
            .build(Tile::Ramp(Ramp::new(dir)))
    }

    pub fn build_road(&mut self, pos: Position, dir: Direction) -> BuildResult {
        self.get_tile_mut(&pos)
            .ok_or(BuildError::InvalidTile)?
            .edit_road(|road| road.connect(dir))
    }

    pub fn is_pos_clear(&self, pos: &Position) -> BuildResult {
        if self.get_tile(pos).ok_or(BuildError::InvalidTile)? == &Tile::Empty {
            Ok(())
        } else {
            Err(BuildError::OccupiedTile)
        }
    }

    pub fn is_area_clear(&self, pos: &Position, size: (i8, i8)) -> BuildResult {
        for x in 0..size.0 {
            for y in 0..size.1 {
                self.is_pos_clear(&(*pos + (x, y).into()))?;
            }
        }
        Ok(())
    }

    pub fn clear(&mut self, pos: &Position) -> BuildResult {
        self.get_tile_mut(pos)
            .ok_or(BuildError::InvalidTile)?
            .clear();

        Ok(())
    }

    pub fn clear_area(&mut self, pos: &Position) -> BuildResult {
        for x in 0..2i8 {
            for y in 0..2i8 {
                self.clear(&(*pos + Direction::from((x, y))))?;
            }
        }
        Ok(())
    }

    pub fn build_building_tile(&mut self, pos: &Position, size: (i8, i8), id: Id) -> BuildResult {
        // let pos = &pos.round_to(2);
        for x in 0..size.0 {
            for y in 0..size.1 {
                self.get_tile_mut(&(*pos + Direction::from((x, y))))
                    .ok_or(BuildError::InvalidTile)?
                    .build(Tile::Building(id))?;
            }
        }
        Ok(())
    }

    pub fn build_road_autoconnect(&mut self, pos: Position) -> BuildResult {
        let pos = pos.round_to(2);

        self.build_two_way_road(pos, Direction::NONE)?;

        // check for roads to connect to
        for dir in Direction::ALL {
            let new_pos = pos + dir * 2;
            if let Some(Tile::Road(_)) = self.get_tile(&new_pos) {
                // if road.connection_count() == 0 {
                    self.build_two_way_road(new_pos, dir)?;
                    self.build_two_way_road(pos, dir)?;
                // }
            }
        }

        Ok(())
    }

    pub fn build_two_way_road(&mut self, pos: Position, dir: Direction) -> BuildResult {
        let pos = pos.round_to(2);

        // we need to be able to overwrite tiles...
        // self.is_area_clear(&pos, (2, 2))?;

        let blueprint = if dir.x == 0 && dir.y == 0 {
            Grid::new_from_string("**\n**")
        } else if dir.is_horizontal() {
            Grid::new_from_string("<<\n>>")
        } else {
            Grid::new_from_string(".^\n.^")
        };

        for (y, row) in blueprint.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                self.build_road(
                    self.pos(x as i16 + pos.x, y as i16 + pos.y),
                    *tile.ground.iter_connections().next().unwrap_or(&Direction::NONE),
                )?
            }
        }

        Ok(())
    }

    pub fn build_one_way_road(&mut self, pos: Position, dir: Direction) -> BuildResult {
        // let pos = &pos.round_to(2);
        for x in 0..2i8 {
            for y in 0..2i8 {
                self.build_road(pos + Direction::from((x, y)), dir)?;
            }
        }
        Ok(())
    }

    pub fn build_building(&mut self, buildings: &mut HashMapId<Building>, building: Building) -> Result<Id, BuildError> {
        self.is_area_clear(&building.pos, BUILDING_SIZE)?;
        let id = buildings.insert(building);
        self.build_building_tile(
            &building.pos,
            BUILDING_SIZE,
            id,
        )?;

        // building.id = self.buildings.id;

        Ok(id)
    }
}

impl Map {
    pub fn clear_area(&mut self, pos: &Position) -> BuildResult {
        let pos = &pos.round_to(2);

        if let Some(Tile::Building(building_id)) = self.grid.get_tile(pos) {
            if let Some(building) = self.buildings.hash_map.remove(building_id) {
                if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
                    if let Some(pos) = city.houses.iter().position(|x| x == building_id) {
                        city.houses.swap_remove(pos);
                    }
                }
            }
        }

        self.grid.clear_area(pos)
    }


}

#[cfg(test)]
mod grid_build_tests {
    use super::Direction;

    use super::*;

    #[test]
    fn test_build() -> BuildResult {
        let mut grid = Grid::new_from_string("___");

        let pos = grid.pos(0, 0);

        grid.build_road(pos, Direction::RIGHT)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        grid.clear(&pos)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('e'));

        grid.build_road(pos, Direction::RIGHT)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        grid.build_road(pos, Direction::UP)?;
        assert_eq!(grid.get_tile(&pos).unwrap(), &Tile::new_from_char('R'));

        Ok(())
    }

    #[test]
    fn test_build_bridge() -> BuildResult {
        let mut grid = Grid::new_from_string("____");

        grid.build_bridge((0, 0).into(), grid.pos(3, 0))?;

        assert_eq!(
            grid.get_tile(&(0, 0).into()).unwrap(),
            &Tile::Ramp(Ramp::new(Direction::LAYER_UP))
        );

        assert_eq!(
            grid.get_tile(&(1, 0, 1).into()).unwrap(),
            &Tile::Road(Road::new_from_char('>').unwrap())
        );

        assert_eq!(
            grid.get_tile(&(2, 0, 1).into()).unwrap(),
            &Tile::Road(Road::new_from_char('>').unwrap())
        );

        assert_eq!(
            grid.get_tile(&(3, 0, 1).into()).unwrap(),
            &Tile::Road(Road::new_from_char('d').unwrap())
        );

        Ok(())
    }

    #[test]
    fn test_build_two_way_road() -> BuildResult {

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::NONE)?;
        assert_eq!(grid, Grid::new_from_string("**__\n**__"));

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::LEFT)?;
        assert_eq!(grid, Grid::new_from_string("<<__\n>>__"));

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::DOWN)?;
        assert_eq!(grid, Grid::new_from_string(".^__\n.^__"));

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_two_way_road(grid.pos(0, 0), Direction::RIGHT)?;
        grid.build_two_way_road(grid.pos(0, 0), Direction::UP)?;
        assert_eq!(grid, Grid::new_from_string("lr__\nLR__"));

        Ok(())
    }

    #[test]
    fn test_build_one_way_road() -> BuildResult {
        let mut grid = Grid::new_from_string("____\n____");
        grid.build_one_way_road(grid.pos(0, 0), Direction::LEFT)?;
        assert_eq!(grid, Grid::new_from_string("<<__\n<<__"));

        let mut grid = Grid::new_from_string("____\n____");
        grid.build_one_way_road(grid.pos(0, 0), Direction::DOWN)?;
        assert_eq!(grid, Grid::new_from_string("..__\n..__"));

        Ok(())
    }
}
