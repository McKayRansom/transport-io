use crate::tile::ConnectionType;

use super::{Grid, Position, Z_BRIDGE};


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


        assert_eq!(grid.tiles[0][0].ground, Tile::new_from_char('u'));
        assert_eq!(grid.tiles[0][1].bridge, Tile::new_from_char('>'));
        assert_eq!(grid.tiles[0][2].bridge, Tile::new_from_char('>'));
        assert_eq!(grid.tiles[0][3].bridge, Tile::new_from_char('d'));
        // assert_eq!(grid.get_tile(grid.pos(0, 0)).)
    }
}