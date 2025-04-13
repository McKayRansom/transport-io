use build::{BuildAction, BuildActionBuilding};
use building::Building;
use city::City;
use company::Company;
use grid::Grid;
use serde::{Deserialize, Serialize};

pub mod build;
pub mod building;
mod city;
pub mod grid;
pub mod path;
pub mod tile;

pub mod draw;
pub mod levels;
mod route;
mod company;
mod generate;
pub use company::Money;
pub mod vehicle;
use route::{Route, RouteDir};

pub mod position;
pub use position::Position;
mod direction;
pub use direction::Direction;
use vehicle::{Status, Vehicle};

use crate::{
    consts::SpawnerColors,
    hash_map_id::{HashMapId, Id},
};

pub const DEFAULT_CITY_ID: Id = 1;

type VehicleHashMap = HashMapId<Vehicle>;
type BuildingHashMap = HashMapId<Building>;
type CityHashMap = HashMapId<City>;
type CompanyHashMap = HashMapId<Company>;

// #[derive(Clone, Debug, Default)]
pub type Tick = u32;

// pub struct VehicleId(Id);
// pub struct BuildingId(Id);
// pub struct CityId(Id);
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Copy, Clone, Debug)]
pub enum Unlocked {
    #[default]
    All,
    Roads,
    OneWayRoads,
    Bridges,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MapMetadata {
    pub is_level: bool,
    pub grow_cities: bool,
    pub level_complete: bool,
    pub level_number: usize,
    pub unlocks: Unlocked,
    // pub level_name: &'static str,
    pub level_hint: Option<String>,
}

impl Default for MapMetadata {
    fn default() -> Self {
        Self {
            is_level: false,
            grow_cities: true,
            level_complete: false,
            level_number: 0,
            unlocks: Unlocked::default(),
            level_hint: None,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct RecentTransactions {
    pub cost: Money,
    pub pos: Position,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Map {
    pub metadata: MapMetadata,
    pub tick: Tick,
    pub grid: Grid,
    pub vehicles: VehicleHashMap,
    cities: CityHashMap,
    pub routes: [Route; 10],
    pub companies: CompanyHashMap,
    #[serde(skip_serializing, skip_deserializing)]
    pub recent_transactions: Vec<RecentTransactions>,
}

impl Map {
    pub fn new_empty(size: (i16, i16)) -> Self {
        // srand(1234);
        Map {
            grid: Grid::new(size),
            ..Default::default()
        }
    }

    pub fn fixup(&mut self) -> Result<(), tile::ReservationError> {
        // Any way to not allow this to be called twice?
        for vehicle in &mut self.vehicles.values_mut() {
            vehicle.fixup(&mut self.grid)?
        }

        Ok(())
    }

    pub fn new_from_string(string: &str) -> Self {
        let mut map = Map {
            grid: Grid::new_from_string(string),
            ..Default::default()
        };

        let city_id = map.new_city(
            Position::new(0, 0) + map.grid.size() / 2,
            "default city".into(),
        );

        // fixup cities
        for (id, building) in map.grid.buildings.hash_map.iter_mut() {
            building.city_id = city_id;
            map.cities
                .hash_map
                .get_mut(&city_id)
                .unwrap()
                .houses
                .push(*id);
        }

        map
    }

    pub fn new_city(&mut self, pos: Position, name: String) -> Id {
        self.cities.insert(City::new(self.cities.id, pos, name))
    }

    #[allow(unused)]
    pub fn get_city(&mut self, id: Id) -> Option<&City> {
        self.cities.hash_map.get(&id)
    }

    pub fn get_city_mut(&mut self, id: Id) -> Option<&mut City> {
        self.cities.hash_map.get_mut(&id)
    }

    pub fn add_vehicle(
        &mut self,
        start: Position,
        end: Id,
        color: SpawnerColors,
        home_id: Option<Id>,
    ) -> Option<Id> {
        let id = self.vehicles.id;
        if let Ok(mut vehicle) = Vehicle::new(id, start, end, &mut self.grid) {
            vehicle.color = color;
            if let Some(id) = home_id {
                vehicle.route_info = Some((id as usize, RouteDir::HouseHackWhy))
            }
            Some(self.vehicles.insert(vehicle))
        } else {
            None
        }
    }

    pub fn reserve_building_id(&mut self) -> Id {
        self.grid.buildings.reserve_id()
    }

    pub fn remove_building(&mut self, id: &Id) -> Option<Building> {
        if let Some(building) = self.grid.buildings.hash_map.remove(id) {
            if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
                if let Some(pos) = city.houses.iter().position(|x| x == id) {
                    city.houses.swap_remove(pos);
                }
            }
            Some(building)
        } else {
            None
        }
    }

    pub fn get_building(&self, id: &Id) -> Option<&Building> {
        self.grid.buildings.hash_map.get(id)
    }

    pub fn insert_building(&mut self, id: Id, building: Building) {
        if let Some(city) = self.cities.hash_map.get_mut(&building.city_id) {
            city.houses.push(id);
        }

        self.grid.buildings.hash_map.insert(id, building);
    }

    fn update_buildings(&mut self) -> bool {
        let mut vehicles_to_add: Vec<Id> = Vec::new();
        let mut all_goals_met = true;

        for (id, building) in &mut self.grid.buildings.hash_map {
            if building.update() {
                vehicles_to_add.push(*id);
            }

            if !building.arrived_goal_met() {
                all_goals_met = false;
            }
        }

        for building_id in vehicles_to_add {
            // generate a random destination
            let start_building = self.grid.buildings.hash_map.get(&building_id).unwrap();

            let end_building_id = self.cities.hash_map[&start_building.city_id].random_house();

            if end_building_id == building_id {
                continue;
            }

            if let Some(destination_building) = self.grid.buildings.hash_map.get(&end_building_id) {
                if !start_building.routes.is_empty() {
                    // try transit
                    let station_id = start_building.routes[0];
                    // cargos_to_add.push(start_building.routes[0]);
                    if let Some(building_station) =
                        self.grid.buildings.hash_map.get_mut(&station_id)
                    {
                        building_station.add_cargo();
                    }
                } else {
                    // spawn car
                    let vehicle_id = self.add_vehicle(
                        start_building.pos,
                        end_building_id,
                        destination_building.color,
                        Some(building_id),
                    );

                    let start_building_again =
                        self.grid.buildings.hash_map.get_mut(&building_id).unwrap();
                    start_building_again.my_vehicle = vehicle_id;
                }
            }
        }

        all_goals_met
    }
    fn update_routes(&mut self) {
        for (i, route) in self.routes.iter_mut().enumerate() {
            let _ = route.update(i, &mut self.grid, &mut self.vehicles);
        }
    }

    fn update_vehicles(&mut self) {
        let mut to_remove: Vec<(Id, Status)> = Vec::new();
        for (id, vehicle) in self.vehicles.hash_map.iter_mut() {
            let status = vehicle.update(&mut self.grid, self.tick);
            if status != Status::EnRoute {
                // TODO: Should busses still be delted if they are hopelessly late?
                if let Some((route_id, dir)) = vehicle.route_info {
                    if dir == RouteDir::HouseHackWhy {
                        to_remove.push((*id, status));
                        continue;
                    }
                    if let Some(route) = self.routes.get(route_id) {
                        if let Some((next_station, dir)) =
                            route.get_next_stop(vehicle.destination, dir)
                        {
                            if let Some(cargo) = vehicle.cargo {
                                if let Some(company) = self.companies.hash_map.get_mut(&vehicle.company) {
                                    company.deposit(cargo);
                                }
                            }
                            vehicle.cargo = self
                                .grid
                                .buildings
                                .hash_map
                                .get_mut(&vehicle.destination)
                                .map(|building| building.pickup_cargo());
                            vehicle.destination = next_station;
                            vehicle.route_info = Some((route_id, dir));
                            // feels like this should be private...
                            vehicle.find_path(&self.grid);
                        }
                    }
                } else {
                    to_remove.push((*id, status));
                }
            }
        }
        for (id, status) in to_remove {
            let vehicle = self.vehicles.hash_map.get_mut(&id).unwrap();
            // let building_id = tile.get_building_id()
            if let Some(route_info) = vehicle.route_info {
                if route_info.1 == RouteDir::HouseHackWhy {
                    if let Some(building) =
                        self.grid.buildings.hash_map.get_mut(&(route_info.0 as u64))
                    {
                        building.update_arrived(status == Status::ReachedDestination);
                        building.my_vehicle = None;
                    }
                }
            }
            self.vehicles.hash_map.remove(&id);
            // self.update_rating(id.1 == Status::ReachedDestination);
        }
    }

    pub fn update(&mut self) -> bool {
        self.tick += 1;

        self.update_vehicles();

        let all_goals_met = self.update_buildings();

        let mut building_to_add: Vec<Building> = Vec::new();
        for city in self.cities.values_mut() {
            if let Some(building) = city.update(&mut self.grid) {
                building_to_add.push(building);
            }
        }

        if self.metadata.grow_cities {
            for building in building_to_add {
                let _ = BuildActionBuilding::new(self, building).execute(self);
            }
        }

        self.update_routes();

        all_goals_met && !self.metadata.level_complete
    }
}

#[cfg(test)]
mod map_tests {

    use super::*;

    #[test]
    fn test_map_rating() {
        Map::new_from_string(">>>>");
        // assert_eq!(map.rating, 1.0);
        // map.update_rating(true);
        // assert_eq!(map.rating, 1.0);
        // map.update_rating(false);
        // assert_eq!(map.rating, 0.9);
    }

    #[test]
    fn test_map_generate() {
        // let mut map = Map::new_from_string("__\n__");

        // map.new_city((0, 0).into(), "test_city".to_string()).unwrap();

        // assert_eq!(map.buildings.len(), 1);

        // assert_eq!(map.vehicles.len(), 0);
        // assert_eq!(map.vehicle_id, 1);

        // for _ in 0..10 * 16 {
        //     map.update_buildings();
        // }

        // assert_eq!(map.vehicles.len(), 1);
        // assert_eq!(map.vehicle_id, 2);
    }
}
