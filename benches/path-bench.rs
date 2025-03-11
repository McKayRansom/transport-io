#![feature(test)]

mod path_bench_tests {
    use pathfinding::{directed::astar::astar, prelude::bfs};

    extern crate test;
    use test::Bencher;
    use transport_io::map::{grid::Grid, Position};

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

        b.iter(||{
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
                |p| grid.road_successors(p).into_iter().map(|(pos, _cost)| pos),
                |p| p == &end,
            )
        })
    }
}