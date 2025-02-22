extern crate test;
use test::Bencher;

use building::BuildingType;
use pathfinding::prelude::{astar, bfs_reach};
use tile::Tile;

use super::*;

type PathCost = u32;
pub type Path = Option<(Vec<Position>, PathCost)>;

impl Grid {
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
            road.get_connections()
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
    use pathfinding::prelude::bfs;

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
            .find_path(((0, 0).into(), Direction::RIGHT), &1)
            .unwrap();
        assert_eq!(path, (vec![(1, 0).into(), (2, 0).into()], 1));
    }

    #[test]
    fn test_path_fail() {
        let grid = Grid::new_from_string("<<1");

        assert!(grid
            .find_path(((0, 0).into(), Direction::RIGHT), &1)
            .is_none());
    }

    #[test]
    fn test_path_house() {
        let grid = Grid::new_from_string("h*h");

        assert_eq!(
            grid.find_path(((0, 0).into(), Direction::RIGHT), &2)
                .unwrap(),
            (vec![(1, 0).into(), (2, 0).into()], 1)
        );
    }

    #[test]
    fn test_path_dead_end() {
        let grid = Grid::new_from_string("h*");

        assert_eq!(
            grid.find_path(((0, 0).into(), Direction::RIGHT), &1)
                .unwrap(),
            (vec![(1, 0).into(), (0, 0).into(),], 1)
        );
    }

    #[test]
    fn test_path_station() {
        let grid = Grid::new_from_string(
            "11<<22
             11>>22",
        );

        assert_eq!(
            grid.find_path(((1, 1).into(), Direction::RIGHT), &2)
                .unwrap(),
            ((2..5).map(|i| (i, 1).into()).collect(), 2)
        );
    }

    fn bench_grid() -> Grid {
        Grid::new_from_string(
            "+++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++
             +++++++++++++++++",
        )
    }

    #[bench]
    fn bench_astar(b: &mut Bencher) {
        let grid = bench_grid();

        b.iter(|| {
            let end = Position::new(15, 15);
            astar(
                &Position::new(7, 7),
                |p| grid.road_successors(p),
                |p| p.distance(&end),
                |p| p == &end,
            )
        })
    }

    #[bench]
    fn bench_bfs(b: &mut Bencher) {
        let grid = bench_grid();

        b.iter(|| {
            let end = Position::new(15, 15);
            bfs(
                &Position::new(7, 7),
                |p| {
                    grid.road_successors(p)
                        .into_iter()
                        .map(|(pos, _cost)| pos)
                },
                |p| p == &end,
            )
        })
    }
}
