use crate::tile::ConnectionType;

use super::{Direction, Grid, Position, Z_BRIDGE};

impl Grid {
    pub fn build_bridge(&mut self, start_pos: Position, end_pos: Position) {
        let (iter, dir) = start_pos.iter_line_to(end_pos, self.size);
        for pos in iter {
            let (build_pos, build_layer) = if pos == start_pos {
                (pos, ConnectionType::Up)
            } else if pos == end_pos {
                (pos.clone_on_layer(Z_BRIDGE), ConnectionType::Down)
            } else {
                (pos.clone_on_layer(Z_BRIDGE), ConnectionType::Road)
            };

            self.get_tile_mut(&build_pos)
                .edit_road(|road| road.connect_layer(dir, build_layer));
        }
    }

    fn build_road(&mut self, pos: &Position, dir: Direction) {
        self.get_tile_mut(pos).edit_road(|road| road.connect(dir));
    }

    pub fn build_two_way_road(&mut self, pos: Position, dir: Direction) {
        let pos = pos.round_to(2);

        let blueprint = if dir.is_horizontal() {
            Grid::new_from_string("<<\n>>")
        } else {
            Grid::new_from_string(".^\n.^")
        };


        for (y, row) in blueprint.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                self.get_tile_mut(&self.pos(x as i16 + pos.x, y as i16 + pos.y))
                    .edit_road(|road|
                        road.connect(tile.ground.iter_connections().next().unwrap()));
            }
        }
   }
}

#[cfg(test)]
mod grid_build_tests {
    use crate::{grid::Direction, tile::Tile};

    use super::*;

    #[test]
    fn test_build() {
        let mut grid = Grid::new_from_string("___");

        grid.get_tile_mut(&grid.pos(0, 0))
            .edit_road(|road| road.connect(Direction::Right));
        assert_eq!(grid.tiles[0][0].ground, Tile::new_from_char('>'));

        grid.get_tile_mut(&grid.pos(0, 0))
            .edit_road(|road| road.connect_layer(Direction::Up, ConnectionType::Road));
        assert_eq!(grid.tiles[0][0].ground, Tile::new_from_char('R'));
    }

    #[test]
    fn test_build_bridge() {
        let mut grid = Grid::new_from_string("____");

        grid.build_bridge(grid.pos(0, 0), grid.pos(3, 0));

        // TODO: Fix this??
        // assert_eq!(grid, Grid::new_from_string("ueee"));
    }

    #[test]
    fn test_build_two_way_road_horizontal() {
        let mut grid = Grid::new_from_string("____\n____");

        grid.build_two_way_road(grid.pos(0, 0), Direction::Left);

        assert_eq!(grid, Grid::new_from_string("<<__\n>>__"));
    }

    #[test]
    fn test_build_two_way_road_vertical() {
        let mut grid = Grid::new_from_string("____\n____");

        grid.build_two_way_road(grid.pos(0, 0), Direction::Down);

        assert_eq!(grid, Grid::new_from_string(".^__\n.^__"));
    }

    #[test]
    fn test_build_two_way_road_intersection() {
        let mut grid = Grid::new_from_string("____\n____");

        grid.build_two_way_road(grid.pos(0, 0), Direction::Right);
        grid.build_two_way_road(grid.pos(0, 0), Direction::Up);

        assert_eq!(grid, Grid::new_from_string("lr__\nLR__"));
    }
}
