use macroquad::prelude::rand;
use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{grid::Grid, vehicle::Vehicle, VehicleHashMap};

// pub struct Passenger {
//     destination: Id,
//     path: Vec<Id>,
//     // do we care about source?
// }

// TODO: Put in building instead?
// pub struct RouteStop {
//     station_id: Id,
//     passengers: Vec<Passenger>,
// }

pub type RouteId = usize;

#[derive(Default, PartialEq, Eq, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum RouteDir {
    #[default]
    Forward,
    Backward,
    HouseHackWhy,
}

impl RouteDir {
    pub fn index(&self) -> i32 {
        match self {
            RouteDir::Forward => 1,
            RouteDir::Backward => -1,
            _ => 0,
        }
    }

    pub fn inverse(&self) -> Self {
        match self {
            RouteDir::Forward => RouteDir::Backward,
            RouteDir::Backward => RouteDir::Forward,
            _ => RouteDir::HouseHackWhy,
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Route {
    pub stops: Vec<Id>,
    pub vehicles: Vec<Id>,
    pub desired_vehicles: usize,
    // is_loop: bool,
}

impl Route {
    pub fn add_stop(&mut self, id: Id, index: usize) {
        self.stops.insert(index, id);
        self.desired_vehicles += 1;
    }

    pub fn get_stop(&mut self, index: usize) -> Option<Id> {
        self.stops.get(index).cloned()
    }

    pub fn get_next_stop(&self, current_stop: Id, route_dir: RouteDir) -> Option<(Id, RouteDir)> {
        let mut i: i32 = if route_dir == RouteDir::Forward {
            0
        } else {
            self.stops.len() as i32 - 1
        };
        while let Some(stop) = self.stops.get(i as usize) {
            if stop == &current_stop {
                if let Some(stop) = self.stops.get((i + route_dir.index()) as usize) {
                    return Some((*stop, route_dir));
                } else if let Some(stop) = self
                    .stops
                    .get((i + route_dir.inverse().index()) as usize)
                {
                    return Some((*stop, route_dir.inverse()));
                } else {
                    // Route of length 1 so just do nothing
                    return None;
                }
            }
            i += route_dir.index();
        }

        None
    }

    pub fn update(
        &mut self,
        index: usize,
        grid: &mut Grid,
        vehicles: &mut VehicleHashMap,
    ) -> Option<()> {
        if self.desired_vehicles <= self.vehicles.len() {
            return None;
        }

        let stop_id = self.get_stop(rand::gen_range(0, self.stops.len()))?;
        let building = grid.buildings.hash_map.get(&stop_id)?;

        let (next_stop, dir) = self.get_next_stop(stop_id, RouteDir::Forward)?;

        let mut vehicle = Vehicle::new(vehicles.id, building.pos, next_stop, grid).ok()?;

        vehicle.route_info = Some((index, dir));
        vehicle.color = SpawnerColors::from_number(index as u64);

        let id = vehicles.insert(vehicle);

        self.vehicles.push(id);
        
        Some(())
    }
}

#[cfg(test)]
mod route_tests {

    use super::*;

    #[test]
    fn test_init() {
        let mut route = Route::default();

        route.add_stop(1234, 0);

        assert_eq!(route.get_stop(0), Some(1234));
        assert_eq!(route.get_stop(1), None);
    }

    #[test]
    fn test_next_stop() {
        let mut route = Route::default();
        route.add_stop(1, 0);
        route.add_stop(2, 1);
        route.add_stop(3, 2);

        assert_eq!(route.get_stop(0).unwrap(), 1);

        assert_eq!(
            route.get_next_stop(1, RouteDir::Forward),
            Some((2, RouteDir::Forward))
        );
        assert_eq!(
            route.get_next_stop(3, RouteDir::Forward),
            Some((2, RouteDir::Backward))
        );
        assert_eq!(
            route.get_next_stop(2, RouteDir::Backward),
            Some((1, RouteDir::Backward))
        );
        assert_eq!(
            route.get_next_stop(1, RouteDir::Backward),
            Some((2, RouteDir::Forward))
        );
    }
}
