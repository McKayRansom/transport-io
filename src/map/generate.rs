use noise::NoiseFn;

use super::{build::{self, BuildAction, BuildActionBuilding, BuildResult}, building::Building, company::Company, tile::Tile, Direction, Map, Position};


impl Map {

    pub fn new_generate(size: (i16, i16)) -> Self {
        let mut map = Self::new_empty(size);

        map.generate().expect("Error generating map");

        map.companies.insert(Company::new());

        map
    }


    fn generate_center_roads(&mut self, pos: Position) -> BuildResult {
        build::action_two_way_road(pos + Direction::LEFT * 5, pos + Direction::RIGHT * 5)
            .execute(self)
            .unwrap();

        build::action_two_way_road(pos + Direction::UP * 5, pos + Direction::DOWN * 5)
            .execute(self)
            .unwrap();

        Ok(())
    }

    fn new_city_generate(&mut self, pos: Position, name: String) -> BuildResult {
        let city_id = self.new_city(pos, name);

        self.generate_center_roads(pos)?;

        for dir in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            BuildActionBuilding::new(self, Building::new_house(pos + dir.into(), city_id))
                .execute(self)?;
        }

        let mut building_to_add: Vec<Building> = Vec::new();
        for _ in 0..10 {
            let city = self.cities.hash_map.get(&city_id).unwrap();
            if let Some(building) = city.grow_building(&self.grid) {
                building_to_add.push(building);
            }
        }
        for building in building_to_add {
            let _ = BuildActionBuilding::new(self, building).execute(self);
        }

        Ok(())
    }

    fn generate(&mut self) -> BuildResult {
        let size: Position = Position::new(0, 0) + self.grid.size();
        let grid_center: Position = size / 2;
        self.new_city_generate(grid_center, "CityVille".to_string())?;

        let perlin_noise = noise::Perlin::new(1234);

        let size = self.grid.size();
        for pos in Position::new(0, 0).iter_area(size) {
            let noise =
                perlin_noise.get([pos.x as f64 / size.x as f64, pos.y as f64 / size.y as f64]);
            if noise < 0.1333 {
                // Tile::Water
                *self.grid.get_tile_mut(&pos).unwrap() = Tile::Water;
            } else if noise < 0.59999 {
                // Tile::Empty
            } else {
                *self.grid.get_tile_mut(&pos).unwrap() = Tile::Mountain;
            };
        }

        Ok(())
    }
}
