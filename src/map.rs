use crate::{grid::{Direction, Grid, Position}, station::Station, tileset::Tileset, vehicle::Vehicle};


const CITY_BLOCK_SIZE: i16 = 10;
const CITY_BLOCK_COUNT: i16 = 2;

pub struct Map {
    pub path_grid: Grid,
    pub vehicles: Vec<Vehicle>,
    pub stations: Vec<Station>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            path_grid: Grid::new(),
            vehicles: Vec::new(),
            stations: Vec::new(),
        }
    }

    pub fn generate_block(&mut self, x: i16, y: i16) {
        // top
        for i in 0..CITY_BLOCK_SIZE {
            self.path_grid.add_allowed(&Position::new(x + i, y), Direction::Right);
            self.path_grid.add_allowed(&Position::new(x + (CITY_BLOCK_SIZE - 1), y + i), Direction::Down);
            self.path_grid.add_allowed(&Position::new(x + i, y + (CITY_BLOCK_SIZE - 1)), Direction::Left);
            self.path_grid.add_allowed(&Position::new(x, y + i), Direction::Up);
        }

        // houses (all for now)
        for i in 0.. CITY_BLOCK_SIZE {
            for j in 0.. CITY_BLOCK_SIZE {
                self.path_grid.add_house(&Position::new(x + i, y + j));
            }
        }
    }

    pub fn generate(&mut self) {
        for i in 0..CITY_BLOCK_COUNT {
            for j in 0..CITY_BLOCK_COUNT {
                self.generate_block(i * CITY_BLOCK_SIZE, j * CITY_BLOCK_SIZE);
            }
        }
    }

    pub fn update(&mut self) {
        //
    }

    pub fn draw(&self, tileset: &Tileset) {

        self.path_grid.draw_tiles(tileset);

        for s in self.stations.iter() {
            s.draw(tileset);
        }

        for s in self.vehicles.iter() {
            s.draw(tileset);
        }
    }

}