use crate::{
    hash_map_id::{HashMapId, Id},
    map::Map,
};

use super::{
    building::{Building, BUILDING_SIZE},
    grid::Grid,
    tile::{Road, Tile},
    Direction, Position,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuildError {
    InvalidTile,
    OccupiedTile,
}

pub type BuildResult = Result<(), BuildError>;

pub trait BuildAction {
    fn execute(&mut self, map: &mut Map) -> BuildResult;
    fn undo(&mut self, map: &mut Map) -> BuildResult;
}

pub struct BuildRoad {
    pos: Position,
    dir: Direction,
    was_empty: bool,
}

impl BuildRoad {
    pub fn new(pos: Position, dir: Direction) -> Self {
        Self {
            pos,
            dir,
            was_empty: false,
        }
    }
}

impl BuildAction for BuildRoad {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        let tile = map
            .grid
            .get_tile_mut(&self.pos)
            .ok_or(BuildError::InvalidTile)?;
        match tile {
            Tile::Empty => {
                self.was_empty = true;
                *tile = Tile::Road(Road::new_connected(self.dir));
                Ok(())
            }
            Tile::Road(road) => {
                self.was_empty = false;
                road.connect(self.dir);
                Ok(())
            }
            _ => Err(BuildError::OccupiedTile),
        }
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        let tile = map
            .grid
            .get_tile_mut(&self.pos)
            .ok_or(BuildError::InvalidTile)?;
        if self.was_empty {
            *tile = Tile::Empty;
            Ok(())
        } else if let Tile::Road(road) = tile {
            road.disconnect(self.dir);
            Ok(())
        } else {
            Err(BuildError::InvalidTile)
        }
    }
}

pub struct BuildActionClearTile {
    pos: Position,
    tile: Option<Tile>,
}

impl BuildActionClearTile {
    pub fn new(pos: Position) -> Self {
        Self { pos, tile: None }
    }
}

impl BuildAction for BuildActionClearTile {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        let tile = map
            .grid
            .get_tile_mut(&self.pos)
            .ok_or(BuildError::InvalidTile)?;
        self.tile = Some(tile.clone());
        *tile = Tile::Empty;
        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        let tile = map
            .grid
            .get_tile_mut(&self.pos)
            .ok_or(BuildError::InvalidTile)?;
        *tile = self.tile.clone().ok_or(BuildError::InvalidTile)?;
        self.tile = None;
        Ok(())
    }
}

pub struct BuildActionClearArea {
    pos: Position,
    clear_actions: [BuildActionClearTile; 4],
    old_building: Option<(Id, Building)>,
}

impl BuildActionClearArea {
    pub fn new(pos: Position) -> Self {
        let pos = pos.round_to(2);
        Self {
            pos,
            clear_actions: [
                BuildActionClearTile::new(pos),
                BuildActionClearTile::new(pos + Direction::RIGHT),
                BuildActionClearTile::new(pos + Direction::DOWN),
                BuildActionClearTile::new(pos + Direction::DOWN_RIGHT),
            ],
            old_building: None,
        }
    }
}

impl BuildAction for BuildActionClearArea {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        if let Some(Tile::Building(building_id)) = map.grid.get_tile(&self.pos) {
            if let Some(building) = map.buildings.hash_map.remove(building_id) {
                if let Some(city) = map.cities.hash_map.get_mut(&building.city_id) {
                    if let Some(pos) = city.houses.iter().position(|x| x == building_id) {
                        city.houses.swap_remove(pos);
                    }
                }
                self.old_building = Some((*building_id, building));
            }
        }

        for action in &mut self.clear_actions {
            action.execute(map)?;
            // TODO: Fix this?
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        for action in &mut self.clear_actions {
            action.undo(map)?;
        }

        if let Some((id, building)) = self.old_building {
            if let Some(city) = map.get_city_mut(building.city_id) {
                city.houses.push(id);
            }

            map.buildings.hash_map.insert(id, building);
        }

        Ok(())
    }
}

pub struct BuildActionList {
    list: Vec<Box<dyn BuildAction>>,
}

impl BuildActionList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn append(&mut self, item: Box<dyn BuildAction>) {
        self.list.push(item);
    }

    fn rollback(&mut self, map: &mut Map, index: usize) {
        for i in 0..index {
            // not sure what we could even do with an error at this point
            let _ = self.list[i].undo(map);
        }
    }

    fn rollforward(&mut self, map: &mut Map, index: usize) {
        for i in 0..index {
            // not sure what we could even do with an error at this point
            let _ = self.list[i].execute(map);
        }
    }
}

impl BuildAction for BuildActionList {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        for (pos, item) in self.list.iter_mut().enumerate() {
            if let Err(err) = item.execute(map) {
                self.rollback(map, pos);
                return Err(err);
            }
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        for (pos, item) in self.list.iter_mut().enumerate() {
            if let Err(err) = item.undo(map) {
                self.rollforward(map, pos);
                return Err(err);
            }
        }

        Ok(())
    }
}

fn build_road_segments_line(action_list: &mut BuildActionList, start_pos: Position, end_pos: Position) {
    let (it, dir) = start_pos.iter_line_to(end_pos);
    for pos in it {
        action_list.append(Box::new(BuildRoad::new(pos, dir)));
    }

    action_list.append(Box::new(BuildRoad::new(end_pos + dir, Direction::NONE)));
}


pub fn action_two_way_road(start_pos: Position, end_pos: Position) -> BuildActionList {
    let mut action_list = BuildActionList::new();

    let dir = start_pos.direction_to(end_pos);
    let start_pos = start_pos.round_to(2);
    let end_pos = end_pos.round_to(2);

    // let's start with the traffic in the current direction
    build_road_segments_line(&mut action_list, start_pos.corner_pos(dir), end_pos.corner_pos(dir));

    // do opposite direction
    let op_dir = dir.inverse();
    build_road_segments_line(&mut action_list, end_pos.corner_pos(op_dir), start_pos.corner_pos(op_dir));

    action_list
}

pub fn action_one_way_road(pos: Position, end_pos: Position) -> BuildActionList {
    let mut action_list = BuildActionList::new();

    let dir = pos.direction_to(end_pos);

    let pos = pos.round_to(2);
    let end_pos = end_pos.round_to(2);

    let pos = pos.corner_pos(dir);
    let end_pos = end_pos.corner_pos(dir);

    build_road_segments_line(&mut action_list, pos, end_pos);
    build_road_segments_line(&mut action_list, pos + dir.rotate_left(), end_pos + dir.rotate_left());

    action_list
}


impl Tile {
    // Left in for now for building houses
    pub fn build(&mut self, tile: Tile) -> BuildResult {
        if *self == Tile::Empty {
            *self = tile;
            Ok(())
        } else {
            Err(BuildError::OccupiedTile)
        }
    }
}

impl Grid {
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


    pub fn build_building(
        &mut self,
        buildings: &mut HashMapId<Building>,
        building: Building,
    ) -> Result<Id, BuildError> {
        self.is_area_clear(&building.pos, BUILDING_SIZE)?;
        let id = buildings.insert(building);
        self.build_building_tile(&building.pos, BUILDING_SIZE, id)?;

        // building.id = self.buildings.id;

        Ok(id)
    }
}

#[cfg(test)]
mod grid_build_tests {
    use crate::map::tile::Ramp;

    use super::Direction;

    use super::*;

    #[test]
    fn test_build() {
        let map = &mut Map::new_blank((2, 2));

        let pos = map.grid.pos(0, 0);

        let mut action = BuildRoad::new(pos, Direction::RIGHT);

        action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));
        action.undo(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::new_from_char('e'));

        action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        let mut clear_action = BuildActionClearTile::new(pos);
        clear_action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::new_from_char('e'));

        action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::new_from_char('>'));

        clear_action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::new_from_char('e'));
    }

    #[test]
    fn test_clear_area() {
        let map = &mut Map::new_from_string(">>\n>>");

        let mut clear_action = BuildActionClearArea::new((0, 0).into());

        clear_action.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("ee\nee"));

        clear_action.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>\n>>"));

        clear_action.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("ee\nee"));

        let city_id = map.new_city((0, 0).into(), "test_city".into());

        map.cities
            .hash_map
            .get_mut(&city_id)
            .unwrap()
            .generate_building((0, 0).into(), &mut map.buildings, &mut map.grid)
            .unwrap();

        assert_ne!(map.grid, Grid::new_from_string("ee\nee"));

        clear_action.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("ee\nee"));

        assert_eq!(
            map.cities.hash_map.get_mut(&city_id).unwrap().houses.len(),
            0
        );

        clear_action.undo(map).unwrap();

        assert_eq!(
            map.cities.hash_map.get_mut(&city_id).unwrap().houses.len(),
            1
        );
    }

    #[test]
    #[ignore = "bridges are broken :("]
    fn test_build_bridge() -> BuildResult {
        let grid = Grid::new_from_string("____");

        // grid.build_bridge((0, 0).into(), grid.pos(3, 0))?;

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
    fn test_build_two_way_road() {
        // TODO
        // let mut grid = Grid::new_from_string("____\n____");
        // grid.build_two_way_road((0, 0).into(), (0, 0).into()).unwrap();
        // assert_eq!(grid, Grid::new_from_string("**__\n**__"));

        let map = &mut Map::new_blank((4, 4));
        let mut action_right = action_two_way_road((0, 0).into(), (2, 0).into());
        action_right.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("*<<<\n>>>*\n____\n____"));
        action_right.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        let mut action_down = action_two_way_road((0, 0).into(), (0, 2).into());
        action_down.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(".*__\n.^__\n.^__\n*^__"));
        action_down.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        action_right.execute(map).unwrap();
        action_down.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(".<<<\nLR>*\n.^__\n*^__"));
        action_down.undo(map).unwrap();
        action_right.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        // test failures
        let mut action_fail_invalid_tile = action_two_way_road((0, 0).into(), (4, 0).into());
        assert_eq!(action_fail_invalid_tile.execute(map).unwrap_err(),
            BuildError::InvalidTile);

        let mut action_fail_occupied = action_two_way_road((0, 0).into(), (2, 0).into());
        map.grid.build_building(&mut map.buildings, Building::new_house((0, 0).into(), 1)).unwrap();
        assert_eq!(action_fail_occupied.execute(map).unwrap_err(),
            BuildError::OccupiedTile);
 
    }

    #[test]
    fn test_build_one_way_road() -> BuildResult {
        let mut map = Map::new_blank((4, 4));
        let mut action_right = action_one_way_road((0, 0).into(), (2, 0).into());
        action_right.execute(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">>>*\n>>>*\n____\n____"));
        action_right.undo(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        let mut action_down = action_one_way_road((0, 0).into(), (0, 2).into());
        action_down.execute(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("..__\n..__\n..__\n**__"));
        action_down.undo(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        Ok(())
    }
}
