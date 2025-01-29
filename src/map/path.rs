
use building::BuildingType;
use pathfinding::prelude::{astar, bfs_reach};
use tile::{Reservation, Tick, Tile};

use super::*;

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReservationError {
    TileInvalid,
    TileReserved,
}

impl Grid {
    pub fn reserve(
        &mut self,
        pos: &Position,
        vehicle_id: Id,
        current: Tick,
        start: Tick,
        end: Tick,
    ) -> Result<Reservation, ReservationError> {
        self.get_tile_mut(pos)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(vehicle_id, *pos, current, start, end)
    }

    pub fn is_reserved(
        &self,
        pos: &Position,
        vehicle_id: Id,
        start: Tick,
        end: Tick,
    ) -> Result<(), ReservationError> {
        self.get_tile(pos)
            .ok_or(ReservationError::TileInvalid)?
            .is_reserved(vehicle_id, start, end)
    }

    pub fn find_path(&self, start: (Position, Direction), end: &Id) -> Path {
        let path_start = start.0 + start.1;

        if !self.get_tile(&path_start)?.is_road() {
            return None;
        }

        let end_building = self.buildings.hash_map.get(end)?;
        if matches!(end_building.building_type, BuildingType::House) {
            // append destination pos (this is already checked to be a road)
            let end_pos_dir = end_building.destination_pos(self)?;
            let end_pos = end_pos_dir.0 + end_pos_dir.1;
            if let Some(mut path) = self.find_road_path(&path_start, &end_pos) {
                path.0.push(end_pos_dir.0);
                path.1 += 1;
                Some(path)
            } else {
                None
            }
        } else {
            self.find_path_to_building(&path_start, end, &end_building.pos)
        }
    }

    pub fn find_road_path(&self, start: &Position, end: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end) / 3,
            |p| p == end,
        )
    }

    pub fn find_path_to_building(&self, start: &Position, end: &Id, end_approx: &Position) -> Path {
        astar(
            start,
            |p| self.road_successors(p),
            |p| p.distance(end_approx) / 3,
            |p| {
                if let Some(tile) = self.get_tile(p) {
                    tile.get_building_id() == Some(*end)
                } else {
                    false
                }
            },
        )
    }

    pub fn road_successors(&self, pos: &Position) -> Vec<(Position, u32)> {
        if let Some(Tile::Road(road)) = self.get_tile(pos) {
            road.get_connections(pos)
                .iter()
                .filter_map(|dir| {
                    let new_pos = *pos + *dir;
                    self.get_tile(&new_pos).map(|tile| (new_pos, tile.cost()))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn iter_reachable<FN>(&self, pos: Position, func: FN)
    where
        FN: Fn(Position),
    {
        for pos in bfs_reach(pos, |pos| {
            self.road_successors(pos)
                .into_iter()
                .map(|(pos, _cost)| pos)
        }) {
            func(pos);
        }
    }
}


#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn test_successors() {
        let grid = Grid::new_from_string(
            ">L>
             _._",
        );
        assert_eq!(
            grid.road_successors(&(0, 0).into()),
            vec![((1, 0).into(), 1)]
        );
        assert_eq!(
            grid.road_successors(&(1, 0).into()),
            vec![((2, 0).into(), 1), ((1, 1).into(), 1)]
        );
        assert_eq!(grid.road_successors(&(0, 1).into()), vec![]);
        assert_eq!(grid.road_successors(&(3, 3).into()), vec![]);
    }

    #[test]
    fn test_path() {
        let grid = Grid::new_from_string(">>1");

        let path = grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .unwrap();
        assert_eq!(path, (vec![grid.pos(1, 0), grid.pos(2, 0)], 1));
    }

    #[test]
    fn test_path_fail() {
        let grid = Grid::new_from_string("<<1");

        assert!(grid
            .find_path((grid.pos(0, 0), Direction::RIGHT), &1)
            .is_none());
    }

    #[test]
    fn test_path_house() {
        let grid = Grid::new_from_string(
            "__<<h_
             _h>>__",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &1)
                .unwrap(),
            ((2..5).map(|i| grid.pos(i, 1)).collect(), 2)
        );
    }

    #[test]
    fn test_path_dead_end() {
        let grid = Grid::new_from_string(
            "h_*<
             __>*",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &1)
                .unwrap(),
            (
                vec![
                    grid.pos(2, 1),
                    grid.pos(3, 1),
                    grid.pos(3, 0),
                    grid.pos(2, 0),
                    grid.pos(1, 0),
                ],
                4
            )
        );
    }

    #[test]
    fn test_path_station() {
        let grid = Grid::new_from_string(
            "11<<22
             11>>22",
        );

        assert_eq!(
            grid.find_path((grid.pos(1, 1), Direction::RIGHT), &2)
                .unwrap(),
            ((2..5).map(|i| grid.pos(i, 1)).collect(), 2)
        );
    }
}
