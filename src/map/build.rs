use crate::{hash_map_id::Id, map::Map};

use super::{
    building::Building,
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
    was_already: bool,
    station: Option<Id>,
}

impl BuildRoad {
    pub fn new(pos: Position, dir: Direction) -> Self {
        Self {
            pos,
            dir,
            was_empty: false,
            was_already: false,
            station: None,
        }
    }

    pub fn new_station(pos: Position, dir: Direction, station: Option<Id>) -> Self {
        Self {
            pos,
            dir,
            was_empty: false,
            was_already: false,
            station,
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
                *tile = Tile::Road(Road::new_connected(self.dir, self.station));
                Ok(())
            }
            Tile::Road(road) => {
                self.was_empty = false;
                self.was_already = road.is_connected(self.dir);
                road.connect(self.dir);

                // Oh god o f
                if self.station.is_some() {
                    road.station = self.station;
                }

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
            if !self.was_already {
                road.disconnect(self.dir);
            }
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

        match tile {
            Tile::Building(_) if map.metadata.is_level => {
                // do not allow destroying tiles in level mode
                Err(BuildError::OccupiedTile)
            }
            _ => {
                self.tile = Some(tile.clone());
                *tile = Tile::Empty;
                Ok(())
            }
        }
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
            clear_actions: Direction::SQUARE.map(|dir| BuildActionClearTile::new(pos + dir)),
            old_building: None,
        }
    }
}

impl BuildAction for BuildActionClearArea {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        // TODO: Move to ClearTile
        if let Some(Tile::Building(building_id)) = map.grid.get_tile(&self.pos).cloned() {
            if let Some(building) = map.remove_building(&building_id) {
                self.old_building = Some((building_id, building));
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

    pub fn new_from_vec(vec: Vec<Box<dyn BuildAction>>) -> Self {
        Self { list: vec }
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

#[derive(Copy, Clone)]
pub enum BuildRoadHeight {
    Level,
    Bridge,
    // Tunnel,
}

pub enum BuildRoadDir {
    Forward,
    Backward,
}

pub struct BuildRoadLane {
    dir: BuildRoadDir,
    // Eventually: Bus-only, train, speed, etc...
}

impl BuildRoadLane {
    pub fn get_build_dir(&self, dir: Direction, index: usize, length: usize) -> Direction {
        if index != length - 1 {
            dir
        } else {
            Direction::NONE
        }
    }

    fn build(
        &self,
        action_list: &mut BuildActionList,
        mut pos: Position,
        mut dir: Direction,
        mut i: usize,
        count: usize,
        height: BuildRoadHeight,
    ) {
        if matches!(self.dir, BuildRoadDir::Backward) {
            i = (count - 1) - i;
            dir = dir.inverse();
        }
        let mut dir = self.get_build_dir(dir, i, count);

        match height {
            BuildRoadHeight::Level => {
                action_list.append(Box::new(BuildRoad::new(pos, dir)));
            }
            BuildRoadHeight::Bridge => {
                match i {
                    0 => {
                        dir = dir + Direction::LAYER_UP;
                    }
                    _ if i == count - 2 => {
                        dir = dir + Direction::LAYER_DOWN;
                        pos = pos + Direction::LAYER_UP;
                    }
                    _ if i == count - 1 => {}
                    _ => {
                        pos = pos + Direction::LAYER_UP;
                    }
                }

                action_list.append(Box::new(BuildRoad::new(pos, dir)));
            } // BuildRoadHeight::Tunnel => todo!(),
        }
    }
}

pub struct RoadBuildOption {
    pub height: BuildRoadHeight,
    pub lanes: &'static [BuildRoadLane],
}

pub fn action_build_road(
    start: Position,
    end: Position,
    options: RoadBuildOption,
) -> BuildActionList {
    let mut action_list = BuildActionList::new();

    let dir = start.direction_to(end);
    let op_dir = dir.inverse();

    // not sure about this...
    if start == end {
        for dir in Direction::SQUARE {
            action_list.append(Box::new(BuildRoad::new(start + dir, Direction::NONE)));
        }
        return action_list;
    }

    // TODO: verify these are in a straight line...
    let start = start.round_to(2).corner_pos(dir);
    let end = end.round_to(2).corner_pos(op_dir);

    let (it, dir) = start.iter_line_to(end);
    let count = it.count;
    for (i, mut pos) in it.enumerate() {
        // go through lanes left to right
        for lane in options.lanes {
            lane.build(&mut action_list, pos, dir, i, count, options.height);
            pos += dir.rotate_right();
        }
    }

    action_list
}

pub const TWO_WAY_ROAD_LANES: &[BuildRoadLane] = &[
    BuildRoadLane {
        dir: BuildRoadDir::Backward,
    },
    BuildRoadLane {
        dir: BuildRoadDir::Forward,
    },
];

pub const ONE_WAY_ROAD_LANES: &[BuildRoadLane] = &[
    BuildRoadLane {
        dir: BuildRoadDir::Forward,
    },
    BuildRoadLane {
        dir: BuildRoadDir::Forward,
    },
];

pub fn action_two_way_road(start_pos: Position, end_pos: Position) -> BuildActionList {
    let options = RoadBuildOption {
        height: BuildRoadHeight::Level,
        lanes: TWO_WAY_ROAD_LANES,
    };

    action_build_road(start_pos, end_pos, options)
}

pub fn action_one_way_road(pos: Position, end_pos: Position) -> BuildActionList {
    let options = RoadBuildOption {
        height: BuildRoadHeight::Level,
        lanes: ONE_WAY_ROAD_LANES,
    };

    action_build_road(pos, end_pos, options)
}

pub struct BuildActionStation {
    road_actions: BuildActionList,
    building: Building,
    building_id: Id,
}

impl BuildActionStation {
    pub fn new(map: &mut Map, pos: Position, building: Building) -> Self {
        let pos = pos.round_to(2);
        let building_id = map.reserve_building_id();
        Self {
            road_actions: BuildActionList::new_from_vec(
                Direction::SQUARE
                    .iter()
                    .map(|dir| -> Box<dyn BuildAction> {
                        Box::new(BuildRoad::new_station(
                            pos + *dir,
                            Direction::NONE,
                            Some(building_id),
                        ))
                    })
                    .collect(),
            ),
            building,
            building_id,
        }
    }
}

impl BuildAction for BuildActionStation {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        map.insert_building(self.building_id, self.building);

        // hack for spawners

        self.road_actions.execute(map)
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        map.remove_building(&self.building_id);
        self.road_actions.undo(map)
    }
}

pub struct BuildActionBuilding {
    building: Building,
    building_id: Option<Id>,
}

impl BuildActionBuilding {
    pub fn new(building: Building) -> Self {
        Self {
            building,
            building_id: None,
        }
    }
}

impl BuildAction for BuildActionBuilding {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        self.building_id = Some(map.add_building(self.building));

        for dir in Direction::SQUARE {
            match map.grid.get_tile_mut(&(self.building.pos + dir)) {
                Some(tile) if tile == &Tile::Empty => {
                    *tile = Tile::Building(self.building_id.unwrap());
                }
                _ => {
                    return Err(BuildError::OccupiedTile);
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        if let Some(building_id) = self.building_id {
            if let Some(_building) = map.remove_building(&building_id) {
                self.building_id = None;
            }
        }

        for dir in Direction::SQUARE {
            *map.grid.get_tile_mut(&(self.building.pos + dir)).unwrap() = Tile::Empty;
        }

        Ok(())
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
}

#[cfg(test)]
mod grid_build_tests {
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

        clear_action.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("ee\nee"));
    }

    #[test]
    fn test_build_two_way_road() {
        let map = &mut Map::new_blank((4, 4));

        let mut action_same_place = action_two_way_road((0, 0).into(), (0, 0).into());
        action_same_place.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("**__\n**__\n____\n____"));
        action_same_place.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        let mut action_right = action_two_way_road((0, 0).into(), (2, 0).into());
        action_right.execute(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("*<<<\n>>>*\n____\n____"));
        action_right.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        // Execute again shouldn't undo
        action_right.execute(map).unwrap();
        action_right.execute(map).unwrap();
        action_right.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("*<<<\n>>>*\n____\n____"));
    }

    #[test]
    fn test_build_two_way_road_again() {
        let map = &mut Map::new_blank((4, 4));

        let mut action_right = action_two_way_road((0, 0).into(), (2, 0).into());
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
        assert_eq!(
            action_fail_invalid_tile.execute(map).unwrap_err(),
            BuildError::InvalidTile
        );
    }

    #[test]
    fn test_build_one_way_road() {
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
    }

    #[test]
    fn test_build_bridge() {
        let mut map = Map::new_blank((4, 4));
        let mut action = action_build_road(
            (0, 0).into(),
            (2, 0).into(),
            RoadBuildOption {
                height: BuildRoadHeight::Bridge,
                lanes: &ONE_WAY_ROAD_LANES,
            },
        );
        action.execute(&mut map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string_layers("}ee*\n}ee*\n____\n____", "e>]e\ne>]e\n____\n____")
        );

        action.undo(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        let mut action = action_build_road(
            (0, 0).into(),
            (2, 0).into(),
            RoadBuildOption {
                height: BuildRoadHeight::Bridge,
                lanes: &TWO_WAY_ROAD_LANES,
            },
        );

        action.execute(&mut map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string_layers("*ee{\n}ee*\n____\n____", "e[<e\ne>]e\n____\n____")
        );
    }

    #[test]
    fn test_build_building() {
        let mut map = Map::new_blank((4, 4));
        let pos: Position = (0, 0).into();
        let city_id = map.new_city(pos, "test".into());
        let building: Building = Building::new_house(pos, city_id);
        let mut action_house = BuildActionBuilding::new(building);

        action_house.execute(&mut map).unwrap();

        assert_eq!(
            map.cities.hash_map.get(&city_id).unwrap().houses[0],
            action_house.building_id.unwrap()
        );

        for dir in Direction::SQUARE {
            assert_eq!(map.grid.get_tile(&(pos + dir)).unwrap(), &Tile::Building(1));
        }

        action_house.undo(&mut map).unwrap();

        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));
        assert!(&action_house.building_id.is_none());

        // Redo
        action_house.execute(&mut map).unwrap();
        for dir in Direction::SQUARE {
            assert_eq!(map.grid.get_tile(&(pos + dir)).unwrap(), &Tile::Building(2));
        }

        // Clear
        let mut clear_action = BuildActionClearArea::new((0, 0).into());

        clear_action.execute(&mut map).unwrap();

        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));
        assert!(map
            .get_building(&action_house.building_id.unwrap())
            .is_none());

        // Undo clear
        clear_action.undo(&mut map).unwrap();
        for dir in Direction::SQUARE {
            assert_eq!(map.grid.get_tile(&(pos + dir)).unwrap(), &Tile::Building(2));
        }
    }
}
