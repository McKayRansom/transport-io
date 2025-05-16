
use macroquad::prelude::rand;
use serde::{Deserialize, Serialize};

use crate::{consts::SpawnerColors, hash_map_id::Id};

use super::{grid::Grid, vehicle::Vehicle, Position, VehicleHashMap};

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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RouteStop {
    pub station: Id,
    pub pos: Position,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Route {
    pub company: Id,
    pub stops: Vec<RouteStop>,
    pub vehicles: Vec<Id>,
    pub desired_vehicles: usize,
    // is_loop: bool,
}

impl Route {
    /*
     * Manhatten distance for now, will need to be grid-path distance in future
     */
    pub fn closest_stop(&self, pos: Position) -> Option<(RouteStop, usize)> {
        let mut closest_stop: Option<(RouteStop, usize)> = None;
        let closest_dist: u32 = 256; // max distance?
        for (index, stop) in self.stops.iter().enumerate() {
            if closest_stop.is_none_or(|(stop, _index)| stop.pos.distance(&pos) < closest_dist) {
                closest_stop = Some((*stop, index));
            }
        }
        closest_stop
    }

    pub fn add_stop(&mut self, new_stop: RouteStop) -> Option<()> {
        if self.stops.contains(&new_stop) {
            // TODO: Special loop case here
            return None;
        }
        // self.stops.insert(index, id);
        let (_closest_stop, closest_index) = self.closest_stop(new_stop.pos)?;
        let distance_prev: Option<u32> = if closest_index > 0 {
            Some(self.stops[closest_index - 1].pos.distance(&new_stop.pos))
        } else {
            None
        };
        let distance_next: Option<u32> = if closest_index + 1 < self.stops.len() {
            Some(self.stops[closest_index + 1].pos.distance(&new_stop.pos))
        } else {
            None
        };
        if distance_prev.is_none() {
            self.stops.insert(0, new_stop);
        } else if distance_next.is_none() {
            self.stops.push(new_stop);
        } else {
            if distance_next.unwrap() < distance_prev.unwrap() {
                self.stops.insert(closest_index + 1, new_stop);
            } else {
                self.stops.insert(closest_index, new_stop);
            }
        }

        self.desired_vehicles += 1;
        Some(())
    }

    pub fn get_stop(&mut self, index: usize) -> Option<RouteStop> {
        self.stops.get(index).cloned()
    }

    pub fn get_next_stop(&self, current_stop: Id, route_dir: RouteDir) -> Option<(Id, RouteDir)> {
        let mut i: i32 = if route_dir == RouteDir::Forward {
            0
        } else {
            self.stops.len() as i32 - 1
        };
        while let Some(stop) = self.stops.get(i as usize) {
            if stop.station == current_stop {
                if let Some(stop) = self.stops.get((i + route_dir.index()) as usize) {
                    return Some((stop.station, route_dir));
                } else if let Some(stop) =
                    self.stops.get((i + route_dir.inverse().index()) as usize)
                {
                    return Some((stop.station, route_dir.inverse()));
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

        let stop = self.get_stop(rand::gen_range(0, self.stops.len()))?;
        let building = grid.buildings.hash_map.get(&stop.station)?;

        let (next_stop, dir) = self.get_next_stop(stop.station, RouteDir::Forward)?;

        let mut vehicle = Vehicle::new(vehicles.id, building.pos, next_stop, grid).ok()?;

        vehicle.company = self.company;
        vehicle.route_info = Some((index, dir));
        vehicle.color = SpawnerColors::from_number(index as u64);

        let id = vehicles.insert(vehicle);

        self.vehicles.push(id);

        Some(())
    }
}

impl Default for Route {
    fn default() -> Self {
        Self {
            company: 1,
            stops: Default::default(),
            vehicles: Default::default(),
            desired_vehicles: Default::default(),
        }
    }
}

#[cfg(test)]
mod route_tests {

    use super::*;

    #[test]
    fn test_init() {
        let mut route = Route::default();

        // route.add_stop(
        //     1234,
        //     RouteStop {
        //         pos: Position::new(0, 0),
        //         station: 0,
        //     },
        // );

        assert_eq!(route.get_stop(0), Some(RouteStop { station: 0, pos: Position::new(0, 0) }));
        assert_eq!(route.get_stop(1), None);
    }

    #[test]
    fn test_next_stop() {
        let mut route = Route::default();
        // route.add_stop(1, Position::new(0, 0));
        // route.add_stop(2, Position::new(1, 0));
        // route.add_stop(3, Position::new(2, 0));

        // assert_eq!(route.get_stop(0).unwrap(), 1);

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
