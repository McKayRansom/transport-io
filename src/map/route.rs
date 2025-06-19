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
        let mut closest_dist: u32 = 256; // max distance?
        for (index, stop) in self.stops.iter().enumerate() {
            let dist = stop.pos.distance(&pos);
            if closest_stop.is_none_or(|(stop, _index)| dist < closest_dist) {
                closest_stop = Some((*stop, index));
                closest_dist = dist;
            }
        }
        closest_stop
    }

    pub fn add_stop(&mut self, new_stop: RouteStop) -> Option<()> {
        if self.stops.contains(&new_stop) {
            // TODO: Special loop case here
            return None;
        }
        self.desired_vehicles += 1;

        if self.stops.len() < 2 {
            self.stops.push(new_stop);
            return Some(());
        }

        // self.stops.insert(index, id);
        let Some((closest_stop, closest_index)) = self.closest_stop(new_stop.pos) else {
            panic!();
        };

        let prev_pos: Option<Position> = if closest_index > 0 {
            Some(self.stops[closest_index - 1].pos)
        } else {
            None
        };
        let next_pos: Option<Position> = if closest_index + 1 < self.stops.len() {
            Some(self.stops[closest_index + 1].pos)
        } else {
            None
        };

        // I had to do this math on paper lol, it's wild
        let distance_if_after = prev_pos.unwrap_or(closest_stop.pos).distance(&closest_stop.pos)
            // + closest_stop.pos.distance(&new_stop.pos) // CANCELS OUT
            + new_stop.pos.distance(&next_pos.unwrap_or(new_stop.pos));

        let distance_if_before = prev_pos.unwrap_or(new_stop.pos).distance(&new_stop.pos)
            // + new_stop.pos.distance(&closest_stop.pos) // CANCELS OUT
            + closest_stop.pos.distance(&next_pos.unwrap_or(closest_stop.pos));

        if distance_if_before < distance_if_after {
            self.stops.insert(closest_index, new_stop);
        } else {
            self.stops.insert(closest_index + 1, new_stop);
        }
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
        id: usize,
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
        vehicle.route_info = Some((id, dir));
        vehicle.color = SpawnerColors::from_number(id as u64);

        let id = vehicles.insert(vehicle);

        self.vehicles.push(id);

        Some(())
    }
    
    pub(crate) fn remove_vehicle(&mut self, id: u64) {
        if let Some(pos) = self.vehicles.iter().position(|&v| v == id) {
            self.vehicles.remove(pos);
        }
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

        assert_eq!(
            route.get_stop(0),
            Some(RouteStop {
                station: 0,
                pos: Position::new(0, 0)
            })
        );
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
