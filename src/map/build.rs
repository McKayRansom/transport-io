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

// TODO: Replace with Something simpler?
pub trait BuildAction {
    fn execute(&mut self, map: &mut Map) -> BuildResult;
    fn undo(&mut self, map: &mut Map) -> BuildResult;
}

pub struct BuildTile {
    pos: Position,
    tile: Tile,
}

impl BuildTile {
    pub fn new(pos: Position, tile: Tile) -> Self {
        Self { pos, tile }
    }
}

impl BuildAction for BuildTile {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        let tile: &mut Tile = map.grid.get_tile_build(&self.pos)?;

        if matches!(tile, Tile::Empty) {
            *tile = self.tile.clone();
            Ok(())
        } else {
            Err(BuildError::OccupiedTile)
        }
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        *map.grid.get_tile_build(&self.pos)? = Tile::Empty;

        Ok(())
    }
}

pub struct BuildRoad {
    pos: Position,
    road: Road,
    old: Tile,
}

impl BuildRoad {
    pub fn new(pos: Position, dirs: &[Direction]) -> Self {
        Self {
            pos,
            road: Road::new_connected(dirs),
            old: Tile::Empty,
        }
    }

    pub fn new_from_road(pos: Position, road: Road) -> Self {
        Self {
            pos,
            road,
            old: Tile::Empty,
        }
    }
}

impl BuildAction for BuildRoad {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        // don't allow building over a bridge entry
        if self.pos.z != 0 {
            if let Some(Tile::Road(road)) = map.grid.get_tile(&(self.pos + Direction::LAYER_DOWN)) {
                for dir in road.connections.iter() {
                    if dir.z != 0 {
                        return Err(BuildError::OccupiedTile);
                    }
                }
            }
        }

        let tile = map.grid.get_tile_build(&self.pos)?;
        self.old = tile.clone();
        match tile {
            Tile::Empty => {
                *tile = Tile::Road(self.road.clone());
                Ok(())
            }
            Tile::Road(road) => {
                // don't allow bridges to cross for now
                if self.pos.z != 0 {
                    return Err(BuildError::OccupiedTile);
                }

                for dir in self.road.connections.iter() {
                    // TODO: Don't allow building a z direction under a bridge...
                    road.connections.connect(dir);
                    if dir.f {
                        // if road.connections.is_connected(dir.inverse()) {
                        // Oh boy oh howdy this doesn't work for the last tile...
                        // How could we mark this pos as not to do this...
                        road.connections.disconnect(dir.inverse());
                        // }
                    }
                }

                Ok(())
            }
            _ => Err(BuildError::OccupiedTile),
        }
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        let tile = map.grid.get_tile_build(&self.pos)?;
        *tile = self.old.clone();

        Ok(())
    }
}

pub struct BuildActionClearArea {
    pos: Position,
    area: Direction,
    old_tiles: Vec<(Position, Tile)>,
    old_buildings: Vec<(Id, Building)>,
}

impl BuildActionClearArea {
    pub fn new(pos: Position, area: Direction) -> Self {
        Self {
            pos,
            area,
            old_tiles: Vec::new(),
            old_buildings: Vec::new(),
        }
    }

    fn clear_tile(&mut self, map: &mut Map, pos: Position) -> BuildResult {
        match map.grid.get_tile_build(&pos)? {
            Tile::Building(_) if map.metadata.is_level => {
                // do not allow destroying buildings in level mode
                return Err(BuildError::OccupiedTile);
            }
            Tile::Building(building_id) => {
                let building_id = *building_id;
                if let Some(building) = map.remove_building(&building_id) {
                    self.old_buildings.push((building_id, building));
                }
            }
            Tile::Road(road) => {
                for dir in road.connections.iter() {
                    if dir.z == 1 {
                        self.clear_bridge(map, pos, dir)?;
                    }
                }
                for dir in Direction::ALL {
                    if let Some(Tile::Road(road)) = map.grid.get_tile_mut(&(pos + dir)) {
                        if road.connections.is_connected(dir.inverse()) {
                            self.old_tiles.push((pos + dir, Tile::Road(road.clone())));
                            road.connections.disconnect(dir.inverse());
                            if road.connections.count() == 0 {
                                *map.grid.get_tile_mut(&(pos + dir)).unwrap() = Tile::Empty;
                            }
                        }
                    }
                    let dir = dir + Direction::LAYER_DOWN;
                    if let Some(Tile::Road(road)) = map.grid.get_tile_mut(&(pos + dir)) {
                        if road.connections.is_connected(dir.inverse()) {
                            self.old_tiles.push((pos + dir, Tile::Road(road.clone())));
                            road.connections.disconnect(dir.inverse());
                            if road.connections.count() == 0 {
                                *map.grid.get_tile_mut(&(pos + dir)).unwrap() = Tile::Empty;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        let tile = map.grid.get_tile_build(&pos)?;
        self.old_tiles.push((pos, tile.clone()));
        *tile = Tile::Empty;
        Ok(())
    }

    fn clear_bridge(&mut self, map: &mut Map, pos: Position, dir: Direction) -> BuildResult {
        let mut bridge_pos = pos + dir;
        let dir = dir.flatten();

        println!("Clearing bridge at {:?} in dir {:?}", pos, dir);

        while let Some(Tile::Road(_road)) = map.grid.get_tile_mut(&bridge_pos) {
            self.clear_tile(map, bridge_pos)?;

            bridge_pos += dir;
        }

        // clear up ramp
        println!("End clearing bridge at {:?}", bridge_pos);

        Ok(())
    }
}

impl BuildAction for BuildActionClearArea {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        for pos in self.pos.iter_area(self.area) {
            // we're not going to stop here...
            let _ = self.clear_tile(map, pos);
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        for (pos, tile) in self.old_tiles.iter().rev() {
            *map.grid.get_tile_build(pos)? = tile.clone();
        }

        self.old_tiles.clear();

        for (id, building) in &self.old_buildings {
            map.insert_building(*id, building.clone());
        }
        self.old_buildings.clear();

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
        for item in self.list[0..index].iter_mut().rev() {
            // not sure what we could even do with an error at this point
            let _ = item.undo(map);
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
        for (pos, item) in self.list.iter_mut().rev().enumerate() {
            if let Err(err) = item.undo(map) {
                self.rollforward(map, pos);
                return Err(err);
            }
        }

        Ok(())
    }
}

impl FromIterator<Box<dyn BuildAction>> for BuildActionList {
    fn from_iter<I: IntoIterator<Item = Box<dyn BuildAction>>>(iter: I) -> Self {
        Self {
            list: Vec::from_iter(iter),
        }
    }
}

#[derive(Copy, Clone)]
pub enum BuildRoadHeight {
    Level,
    Bridge,
}

pub struct RoadBuildOption {
    pub height: BuildRoadHeight,
    pub one_way: bool,
}

fn build_lane(
    action_list: &mut BuildActionList,
    mut pos: Position,
    dir: Direction,
    i: usize,
    count: usize,
    options: &RoadBuildOption,
) {
    let dirs: &[Direction] = match options.height {
        BuildRoadHeight::Level => {
            if i == 0 {
                if options.one_way {
                    &[dir.flag()]
                } else {
                    &[dir]
                }
            } else if i != count - 1 {
                if options.one_way {
                    &[dir.flag()]
                } else {
                    &[dir, dir.inverse()]
                }
            } else {
                if options.one_way {
                    &[]
                } else {
                    &[dir.inverse()]
                }
            }
        }
        BuildRoadHeight::Bridge => {
            match i {
                0 => &[dir + Direction::LAYER_UP],

                _ if i == count - 1 => {
                    // Last bridge pos is at ground level
                    &[dir.inverse() + Direction::LAYER_UP]
                }
                _ => {
                    pos += Direction::LAYER_UP;
                    &[
                        if i == 1 {
                            dir.inverse() + Direction::LAYER_DOWN
                        } else {
                            dir.inverse()
                        },
                        if i == count - 2 {
                            dir + Direction::LAYER_DOWN
                        } else {
                            dir
                        },
                    ]
                }
            }
        }
    };
    action_list.append(Box::new(BuildRoad::new(pos, dirs)));
}

pub fn action_build_road(
    start: Position,
    end: Position,
    options: RoadBuildOption,
) -> BuildActionList {
    let mut action_list = BuildActionList::new();

    // not sure about this...
    if start == end {
        action_list.append(Box::new(BuildRoad::new(start, &[Direction::NONE])));
        return action_list;
    }

    let (it, dir) = start.iter_line_to(end);
    let count = it.count;

    // BUG: We need to throw an error here but we can't...
    // if matches!(options.height, BuildRoadHeight::Bridge) && count < 3 {
    // OOOOOOFFFFFFF
    // return action_list;
    // }
    for (i, pos) in it.enumerate() {
        build_lane(&mut action_list, pos, dir, i, count, &options);
    }

    action_list
}

pub fn action_two_way_road(start_pos: Position, end_pos: Position) -> BuildActionList {
    let options = RoadBuildOption {
        height: BuildRoadHeight::Level,
        one_way: false,
    };

    action_build_road(start_pos, end_pos, options)
}

pub fn action_one_way_road(pos: Position, end_pos: Position) -> BuildActionList {
    let options = RoadBuildOption {
        height: BuildRoadHeight::Level,
        one_way: true,
    };

    action_build_road(pos, end_pos, options)
}

pub struct BuildActionStation {
    pos: Position,
    building: Building,
    building_id: Id,
}

impl BuildActionStation {
    pub fn new(map: &mut Map, pos: Position, building: Building) -> Self {
        Self {
            pos,
            building,
            building_id: map.reserve_building_id(),
        }
    }
}

impl BuildAction for BuildActionStation {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        if let Tile::Road(road) = map.grid.get_tile_build(&self.pos)? {
            if road.station.is_some() {
                return Err(BuildError::OccupiedTile);
            }
            road.station = Some(self.building_id);
            map.insert_building(self.building_id, self.building.clone());

            // TODO: Function and undo this...
            for pos in (self.pos + Direction::UP_LEFT * 2).iter_area(Direction::DOWN_RIGHT * 5) {
                if let Some(&Tile::Building(building_id)) = map.grid.get_tile(&pos) {
                    if let Some(building) = map.grid.buildings.hash_map.get_mut(&building_id) {
                        building.routes.push(self.building_id);
                    }
                }
            }
            Ok(())
        } else {
            Err(BuildError::InvalidTile)
        }
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        map.remove_building(&self.building_id);
        if let Tile::Road(road) = map.grid.get_tile_build(&self.pos)? {
            road.station = None;
        }
        Ok(())
    }
}

pub struct BuildActionBuilding {
    building: Building,
    building_id: Id,
}

impl BuildActionBuilding {
    pub fn new(map: &mut Map, building: Building) -> Self {
        Self {
            building,
            building_id: map.reserve_building_id(),
        }
    }
}

impl BuildAction for BuildActionBuilding {
    fn execute(&mut self, map: &mut Map) -> BuildResult {
        // for dir in Direction::SQUARE {
        BuildTile::new(self.building.pos, Tile::Building(self.building_id)).execute(map)?;
        // }

        let mut building = self.building.clone();
        for pos in (self.building.pos + Direction::UP_LEFT * 2).iter_area(Direction::DOWN_RIGHT * 5)
        {
            if let Some(Tile::Road(road)) = map.grid.get_tile(&pos) {
                if let Some(station_id) = road.station {
                    building.routes.push(station_id);
                }
            }
        }

        map.insert_building(self.building_id, building);

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> BuildResult {
        map.remove_building(&self.building_id);

        // for dir in Direction::SQUARE {
        BuildTile::new(self.building.pos, Tile::Building(self.building_id)).undo(map)?;
        // }

        Ok(())
    }
}

pub struct Clipboard {
    tiles: Vec<(Direction, Road)>,
}

impl Clipboard {
    pub fn new(map: &Map, start_pos: Position, area: Direction) -> Self {
        let mut tiles: Vec<(Direction, Road)> = Vec::new();
        for pos in start_pos.iter_area(area) {
            match map.grid.get_tile(&pos) {
                Some(Tile::Road(road)) => tiles.push((pos - start_pos, road.clone())),
                _ => {},
            }
            match map.grid.get_tile(&(pos + Direction::LAYER_UP)) {
                Some(Tile::Road(road)) => tiles.push(((pos - start_pos) + Direction::LAYER_UP, road.clone())),
                _ => {},
            }
        }
        Self {
            tiles
        }
    }

    pub fn paste(&self, pos: Position) -> BuildActionList {
        self.tiles
            .iter()
            .map(|(dir, road)| -> Box<dyn BuildAction> {
                Box::new(BuildRoad::new_from_road(pos + *dir, road.clone()))
            })
            .collect()
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

    pub fn is_area_clear(&self, pos: &Position, size: Direction) -> BuildResult {
        for x in 0..size.x {
            for y in 0..size.y {
                self.is_pos_clear(&(*pos + (x, y).into()))?;
            }
        }
        Ok(())
    }

    fn get_tile_build(&mut self, pos: &Position) -> Result<&mut Tile, BuildError> {
        self.get_tile_mut(pos).ok_or(BuildError::InvalidTile)
    }
}

#[cfg(test)]
mod grid_build_tests {
    use crate::hash_map_id::HashMapId;
    use super::Direction;
    use super::*;

    #[test]
    fn test_build() {
        let map = &mut Map::new_empty((2, 2));

        let pos = Position::new(0, 0);

        let mut action = BuildRoad::new(pos, &[Direction::RIGHT]);

        action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &('>').into());
        action.undo(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &('e').into());

        action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &('>').into());

        let mut clear_action = BuildActionClearArea::new(pos, (1, 1).into());
        clear_action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &('e').into());

        action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &('>').into());

        clear_action.execute(map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &('e').into());
    }

    #[test]
    fn test_clear_area() {
        let map = &mut Map::new_from_string(">>\n>>");

        let mut clear_action = BuildActionClearArea::new((0, 0).into(), (2, 2).into());

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
        let map = &mut Map::new_empty((2, 2));

        let mut action_same_place = action_two_way_road((0, 0).into(), (0, 0).into());
        action_same_place.execute(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "*e
                 ee"
            )
        );
        action_same_place.undo(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "ee
                 ee"
            )
        );

        let mut action_right = action_two_way_road((0, 0).into(), (1, 0).into());
        action_right.execute(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "><
                 ee"
            )
        );
        action_right.undo(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "ee
                 ee"
            )
        );

        // Execute again shouldn't undo
        action_right.execute(map).unwrap();
        action_right.execute(map).unwrap();
        action_right.undo(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "><
                 ee"
            )
        );
    }

    #[test]
    fn test_build_two_way_road_again() {
        let map = &mut Map::new_empty((4, 4));

        let mut action_right = action_two_way_road((0, 0).into(), (2, 0).into());
        let mut action_down = action_two_way_road((0, 0).into(), (0, 2).into());
        action_down.execute(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                ".eee
             |eee
             ^eee
             eeee"
            )
        );
        action_down.undo(map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        action_right.execute(map).unwrap();
        action_down.execute(map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "L-<e
                 |eee
                 ^eee
                 eeee"
            )
        );
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
        let mut map = Map::new_empty((4, 4));
        let mut action_one_way = action_one_way_road((0, 0).into(), (2, 0).into());
        action_one_way.execute(&mut map).unwrap();
        assert_eq!(
            map.grid,
            Grid::new_from_string(
                "dd*_
             ____
             ____
             ____"
            )
        );
        action_one_way.undo(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        // overwrite two-way road with one-way road
        let mut action_two_way = action_two_way_road((0, 0).into(), (2, 0).into());
        action_two_way.execute(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string(">-<_\n____\n____\n____"));
        action_one_way.execute(&mut map).unwrap();
        // TODO: Dead end overwriting doesn't work right...
        // assert_eq!(map.grid, Grid::new_from_string("dd*_\n____\n____\n____"));
        action_one_way.undo(&mut map).unwrap();
        action_two_way.undo(&mut map).unwrap();

        // let mut action_down = action_one_way_road((0, 0).into(), (0, 2).into());
        // action_down.execute(&mut map).unwrap();
        // assert_eq!(map.grid, Grid::new_from_string("..__\n..__\n..__\n..__"));
        // action_down.undo(&mut map).unwrap();
        // assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));
    }

    #[test]
    fn test_build_bridge() {
        let mut map = Map::new_empty((5, 1));
        let mut action = action_build_road(
            (0, 0).into(),
            (4, 0).into(),
            RoadBuildOption {
                height: BuildRoadHeight::Bridge,
                one_way: false,
            },
        );
        action.execute(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string_layers("}___{", "_[-]__"));

        action.undo(&mut map).unwrap();
        assert_eq!(map.grid, Grid::new_from_string("_____"));

        let mut map = Map::new_empty((5, 5));
        let mut action = action_build_road(
            (0, 1).into(),
            (2, 1).into(),
            RoadBuildOption {
                height: BuildRoadHeight::Bridge,
                one_way: false,
            },
        );

        action.execute(&mut map).unwrap();
        // assert_eq!(map.grid, Grid::new_from_string_layers("})e)*_", "e>>]e__"));
        // assert_eq!(map.grid, Grid::new_from_string_layers("<(({__", "e>]e__"));

        // don't allow bridges to cross
        let mut action = action_build_road(
            (1, 0).into(),
            (1, 2).into(),
            RoadBuildOption {
                height: BuildRoadHeight::Bridge,
                one_way: false,
            },
        );

        assert!(action.execute(&mut map).is_err());

        // destroy the bridge!
        BuildActionClearArea::new((0, 0).into(), (0, 0).into())
            .execute(&mut map)
            .unwrap();

        // assert_eq!(map.grid, Grid::new_from_string("eeeeee"));
    }

    #[test]
    fn test_build_building() {
        let mut map = Map::new_empty((4, 4));
        let pos: Position = (0, 0).into();
        let city_id = map.new_city(pos, "test".into());
        let building: Building = Building::new_house(pos, city_id);
        let mut action_house = BuildActionBuilding::new(&mut map, building);

        action_house.execute(&mut map).unwrap();

        assert_eq!(
            map.cities.hash_map.get(&city_id).unwrap().houses[0],
            action_house.building_id
        );
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::Building(1));

        action_house.undo(&mut map).unwrap();

        map.grid.buildings = HashMapId::new();
        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        // Redo
        action_house.execute(&mut map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::Building(1));

        // Clear
        let mut clear_action = BuildActionClearArea::new((0, 0).into(), (2, 2).into());

        clear_action.execute(&mut map).unwrap();

        assert_eq!(map.grid, Grid::new_from_string("____\n____\n____\n____"));

        // Undo clear
        clear_action.undo(&mut map).unwrap();
        assert_eq!(map.grid.get_tile(&pos).unwrap(), &Tile::Building(1));
    }

    #[test]
    fn test_clipboard() {
        let mut map = Map::new_from_string("><__");

        let clipboard = Clipboard::new(&map, (1, 0).into(), (1, 0).into());

        let mut paste_action = clipboard.paste((2, 0).into());

        paste_action.execute(&mut map).unwrap();

        assert_eq!(map.grid, Grid::new_from_string("><<_"));
        paste_action.undo(&mut map).unwrap();

        assert_eq!(map.grid, Grid::new_from_string("><__"));
    }

    // TODO: Test case for reversing BuildList in correct order
}
