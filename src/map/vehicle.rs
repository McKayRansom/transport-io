use macroquad::color::BLUE;
use macroquad::color::RED;
use macroquad::color::WHITE;
use macroquad::math::Rect;
use serde::Deserialize;
use serde::Serialize;

use crate::consts::SpawnerColors;
use crate::tileset::Sprite;
use crate::tileset::Tileset;
use crate::hash_map_id::Id;

use super::grid::Grid;
use super::grid::Path;
use super::grid::ReservationError;
use super::grid::GRID_CELL_SIZE;
use super::tile::Reservation;
use super::tile::Tile;
use super::Direction;
use super::Position;

const SPEED_PIXELS_PER_TICK: i8 = 2; 
const SPEED_TICKS_PER_TILE: i16 = GRID_CELL_SIZE.0 as i16 / SPEED_PIXELS_PER_TICK as i16;
const HOPELESSLY_LATE_PERCENT: f32 = 0.5;

const CAR_SPRITE: Sprite = Sprite::new(0, 1);
const CAR_SHADOW_SPRITE: Sprite = Sprite::new(0, 2);

#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Id,
    // pathing
    path: Path,
    reserved: Vec<Reservation>,
    path_index: usize,
    path_time_ticks: u32,
    elapsed_ticks: u32,
    pub destination: Position,
    // position
    pub pos: Position,
    lag_pos_pixels: Direction,
    pub dir: Direction,
    // This is an optimization and doesn't need to be saved
    #[serde(skip_serializing, skip_deserializing)]
    blocking_tile: Option<Position>,

    // this could be calculated from destination
    pub color: SpawnerColors,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd)]
pub enum Status {
    EnRoute,
    ReachedDestination,
    HopelesslyLate,
}

pub enum ReservePathError {
    InvalidPath,
    Blocking(Position),
}

impl Vehicle {
    pub fn new(
        pos: Position,
        id: Id,
        destination: Position,
        grid: &mut Grid,
    ) -> Result<Self, ReservationError> {
        let reservation = grid
            .get_tile_mut(&pos)
            .ok_or(ReservationError::TileInvalid)?
            .reserve(id, pos)?;

        let mut vehicle = Vehicle {
            id,
            path: None,
            path_time_ticks: 0,
            path_index: 0,
            elapsed_ticks: 0,
            pos,
            lag_pos_pixels: Direction::NONE,
            dir: Direction::RIGHT,
            blocking_tile: None,
            color: SpawnerColors::Blue,
            destination,
            reserved: vec![reservation],
        };

        vehicle.find_path(grid);

        Ok(vehicle)
    }

    pub fn fixup(&mut self, grid: &mut Grid) -> Result<(), ReservationError> {
        // Fix serialization
        for reservation in &mut self.reserved {
            *reservation = grid.reserve(&reservation.pos, self.id)?
        }

        Ok(())
    }

    fn reserve(
        path_grid: &mut Grid,
        vehicle_id: Id,
        position: Position,
        reserved: &mut Vec<Reservation>,
    ) -> Result<(), ReservationError> {
        let reservation = path_grid.reserve(&position, vehicle_id)?;
        reserved.push(reservation);
        Ok(())
    }

    fn reserve_path(&self, grid: &mut Grid) -> Result<Vec<Reservation>, ReservePathError> {
        // TODO: Move to grid

        let should_yield = grid
            .get_tile(&self.pos)
            .ok_or(ReservePathError::InvalidPath)?
            .should_yield();

        let (path, _cost) = self.path.as_ref().ok_or(ReservePathError::InvalidPath)?;

        let mut reserved = Vec::<Reservation>::new();

        // for pos in &path[self.path_index + 1..] {
        if let Some(pos) = path.get(self.path_index + 1) {
            // TODO Make function
            match Vehicle::reserve(grid, self.id, *pos, &mut reserved) {
                Ok(_) => {
                    if let Some(yield_to_pos) =
                        grid.should_we_yield_when_entering(should_yield, pos)
                    {
                        return Err(ReservePathError::Blocking(yield_to_pos));
                    }
                    // Fall through
                }
                Err(ReservationError::TileInvalid) => {
                    return Err(ReservePathError::InvalidPath);
                }
                Err(ReservationError::TileReserved) => {
                    return Err(ReservePathError::Blocking(*pos));
                }
            }
        }

        Ok(reserved)
    }

    fn update_speed(&mut self) {

        // TODO: fix bugs with x not a multiple of SPEED_PIXELS...
        // match self.lag_pos_pixels.x.cmp(&0)
        if self.lag_pos_pixels.x > 0 {
            self.lag_pos_pixels.x -= SPEED_PIXELS_PER_TICK;
        } else if self.lag_pos_pixels.x < 0 {
            self.lag_pos_pixels.x += SPEED_PIXELS_PER_TICK;
        }
        if self.lag_pos_pixels.y > 0 {
            self.lag_pos_pixels.y -= SPEED_PIXELS_PER_TICK;
        } else if self.lag_pos_pixels.y < 0 {
            self.lag_pos_pixels.y += SPEED_PIXELS_PER_TICK;
        }
        if self.lag_pos_pixels.z > 0 {
            self.lag_pos_pixels.z -= SPEED_PIXELS_PER_TICK;
        } else if self.lag_pos_pixels.z < 0 {
            self.lag_pos_pixels.z += SPEED_PIXELS_PER_TICK;
        }
    }

    fn find_path(&mut self, grid: &mut Grid) -> bool {
        self.path = grid.find_path(&self.pos, &self.destination);

        if let Some(path) = &self.path {
            self.path_time_ticks = path.1 * SPEED_TICKS_PER_TILE as u32;
            self.path_index = 0;
        }
        self.path.is_some()
    }

    fn get_next_pos(&mut self, grid: &mut Grid) -> Option<Position> {
        match self.reserve_path(grid) {
            Ok(reserved) => {
                self.reserved = reserved;
                if let Some(path) = self.path.as_ref() {
                    let pos = path.0[self.path_index + 1];
                    self.path_index += 1;
                    Some(pos)
                } else {
                    None
                }
            }
            Err(ReservePathError::InvalidPath) => {
                self.find_path(grid);
                // we're pretty well screwed if this happens so maybe don't do this??
                // TODO: Don't do this, just unreserve when we find a path!
                let _ = Vehicle::reserve(grid, self.id, self.pos, &mut self.reserved);
                None
            }
            Err(ReservePathError::Blocking(blocking_pos)) => {
                self.blocking_tile = Some(blocking_pos);
                let _ = Vehicle::reserve(grid, self.id, self.pos, &mut self.reserved);
                // grid.reserve_position(&self.pos, self.id);
                None
            }
        }
    }

    fn update_position(&mut self, path_grid: &mut Grid) -> Status {
        if let Some(blocking_tile) = self.blocking_tile {
            if let Some(Tile::Road(road)) = path_grid.get_tile(&blocking_tile) {
                if road.reserved.is_reserved() {
                    // don't bother
                    return Status::EnRoute;
                }
            }
        }
        self.blocking_tile = None;

        if self.pos == self.destination {
            return Status::ReachedDestination;
        }

        self.reserved.clear();

        if let Some(next_pos) = self.get_next_pos(path_grid) {
            self.dir = next_pos - self.pos;
            // self.dir.z = -self.dir.z;
            self.lag_pos_pixels = self.dir * -GRID_CELL_SIZE.0 as i8;
            self.update_speed();
            self.pos = next_pos;
        }

        Status::EnRoute
    }

    // 0.5 = 50% late
    // 1 = on time exactly
    // 1.5 = 50% early
    pub fn trip_late(&self) -> f32 {
        if let Some(path) = &self.path {
            let tiles_elapsed =
                (self.elapsed_ticks.saturating_sub(1) / SPEED_TICKS_PER_TILE as u32) + 1;
            let tiles_expected = path.1;

            let elapsed_percent = tiles_elapsed as f32 / tiles_expected as f32;

            let completed_percent = self.trip_completed_percent();
            // println!("elapsed: {tiles_elapsed}, expected: {} percent: {completed_percent}", tiles_expected);

            if completed_percent > 0. {
                1. - (elapsed_percent - completed_percent)
            } else {
                1.
            }
        } else {
            1.
        }
    }

    pub fn trip_completed_percent(&self) -> f32 {
        if let Some(path) = &self.path {
            self.path_index.max(0) as f32 / (path.0.len() - 1).max(1) as f32
        } else {
            1.
        }
    }

    pub fn update(&mut self, path_grid: &mut Grid) -> Status {
        self.elapsed_ticks += 1;
        if self.trip_late() < HOPELESSLY_LATE_PERCENT {
            Status::HopelesslyLate
        } else if self.lag_pos_pixels != Direction::NONE {
            self.update_speed();
            Status::EnRoute
        } else {
            self.update_position(path_grid)
        }
    }

    pub fn draw(&self, tileset: &Tileset) {
        let mut rect = Rect::from(self.pos);
        rect.x += self.lag_pos_pixels.x as f32;
        rect.y += self.lag_pos_pixels.y as f32; // - (self.lag_pos_pixels.z as f32) / (GRID_CELL_SIZE.0 / 10.);

        // let vehicle_red = Color::from_hex(0xf9524c);
        // let vehicle_blue = Color::from_hex(0xa0dae8);
        // let vehicle_yellow = Color::from_hex(0xf8c768);

        // let mut color = vehicle_blue;

        // if self.path.is_none() {
        //     color = vehicle_red;
        // // } else if self.blocking_tile.is_some() {
        // } else if self.trip_late() < 0.75 {
        //     color = vehicle_yellow;
        // }
        // draw shadow
        let mut shadow_rect = rect;
        shadow_rect.x += 2.;
        shadow_rect.y += 2.;
        tileset.draw_tile(CAR_SHADOW_SPRITE, WHITE, &shadow_rect, self.dir.to_radians());

        tileset.draw_tile(CAR_SPRITE, self.color.color(), &rect, self.dir.to_radians());
    }

    pub(crate) fn draw_detail(&self, tileset: &Tileset) {
        // draw reserved
        let mut reserved_path_color = RED;
        reserved_path_color.a = 0.3;
        // for pos in self.reserved {
        //     tileset.draw_rect(&Rect::from(pos), reserved_path_color);
        // }

        let mut path_color = BLUE;
        path_color.a = 0.3;
        if let Some(path) = self.path.as_ref() {
            for pos in &path.0 {
                tileset.draw_rect(&Rect::from(*pos), path_color);
            }
        }
    }
}

#[cfg(test)]
mod vehicle_tests {

    use super::*;

    fn reserve(grid: &mut Grid, pos: Position) -> Result<Reservation, ReservationError> {
        grid.get_tile_mut(&pos).unwrap().reserve(1234, pos)
    }

    #[test]
    fn test_init() {
        let mut grid = Grid::new_from_string(">>>>");
        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(3, 0);
        let vehicle = Vehicle::new(start_pos, 0, end_pos, &mut grid).unwrap();

        assert_eq!(
            reserve(&mut grid, start_pos).unwrap_err(),
            ReservationError::TileReserved
        );

        assert!(Vehicle::new(start_pos, 2, end_pos, &mut grid).is_err());

        drop(vehicle)
    }

    #[test]
    fn test_status() {
        let mut grid = Grid::new_from_string(">>>>");
        let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, grid.pos(3, 0), &mut grid).unwrap();

        // let
        // let reservation = grid.get_tile_mut(&grid.pos(1, 0)).reserve(1).unwrap();

        assert_eq!(vehicle.update(&mut grid), Status::EnRoute);

        // drop(reservation);
    }

    #[test]
    fn test_late() {
        let mut grid = Grid::new_from_string(">>>>");
        let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, grid.pos(3, 0), &mut grid).unwrap();

        vehicle.update(&mut grid);

        vehicle.elapsed_ticks = SPEED_TICKS_PER_TILE as u32 + 1;
        assert_eq!(vehicle.trip_late(), 0.6666666);

        vehicle.elapsed_ticks = (SPEED_TICKS_PER_TILE * 2) as u32 + 1;
        assert_eq!(vehicle.trip_late(), 0.33333337);
    }

    #[test]
    fn test_trip() {
        let mut grid = Grid::new_from_string(">>>>");
        let destination = grid.pos(3, 0);
        let mut vehicle = Vehicle::new(grid.pos(0, 0), 0, destination, &mut grid).unwrap();

        let trip_length: u32 = 3;
        let trip_time = SPEED_TICKS_PER_TILE as u32 * trip_length;

        assert_eq!(vehicle.path_time_ticks, trip_time);

        assert_eq!(vehicle.trip_completed_percent(), 0.);

        assert_eq!(vehicle.trip_late(), 1.0);
        // grid.reserve_position(&(1, 0).into(), 1);

        // assert_eq!(vehicle.update(&mut grid), Status::EnRoute);

        for i in 0..(trip_length * SPEED_TICKS_PER_TILE as u32) {
            assert_eq!(
                vehicle.update(&mut grid),
                Status::EnRoute,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.path_index,
                1 + (i / (SPEED_TICKS_PER_TILE as u32)) as usize,
                "Failed on tick {i}"
            );
            assert_eq!(vehicle.elapsed_ticks, i + 1);
            assert_eq!(
                vehicle.trip_completed_percent(),
                ((i + SPEED_TICKS_PER_TILE as u32) / SPEED_TICKS_PER_TILE as u32) as f32 / trip_length as f32,
                "Failed on tick {i}"
            );
            assert_eq!(
                vehicle.trip_late(),
                1.0,
                "Failed on tick {i} %{}",
                vehicle.trip_completed_percent()
            );
        }

        println!("Vehicle : {:?}", vehicle.blocking_tile);
        assert_eq!(vehicle.update(&mut grid), Status::ReachedDestination);
        assert_eq!(vehicle.pos, destination);
        assert_ne!(vehicle.trip_late(), 1.0);
    }

    #[test]
    fn test_reserved() {
        let mut grid = Grid::new_from_string(">>>>");

        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(3, 0);

        let mut vehicle = Vehicle::new(start_pos, 0, end_pos, &mut grid).unwrap();

        assert_eq!(
            Vehicle::reserve(&mut grid, 12, end_pos, &mut vehicle.reserved),
            Ok(())
        );

        assert_eq!(
            Vehicle::reserve(&mut grid, 12, end_pos, &mut vehicle.reserved),
            Err(ReservationError::TileReserved)
        );
    }

    #[test]
    fn test_update_speed() {
        let mut grid = Grid::new_from_string(">>>>");
        let start_pos = grid.pos(0, 0);
        let end_pos = grid.pos(3, 0);
        let mut vehicle = Vehicle::new(start_pos, 0, end_pos, &mut grid).unwrap();

        vehicle.lag_pos_pixels.x = SPEED_PIXELS_PER_TICK;
        vehicle.lag_pos_pixels.y = SPEED_PIXELS_PER_TICK;
        vehicle.lag_pos_pixels.z = SPEED_PIXELS_PER_TICK;

        vehicle.update_speed();

        assert_eq!(vehicle.lag_pos_pixels, Direction::NONE);
    }

    #[test]
    fn test_blocking_tile() {}

    #[test]
    fn test_yield() {
        let mut grid = Grid::new_from_string(
            "\
            >>>>>
            _h___",
        );

        // println!("grid: {:?}", &grid);

        let start_pos = grid.pos(1, 1);
        let yield_to_pos = grid.pos(0, 0);
        // let intersection_pos = grid.pos(1, 0);
        let mut vehicle = Vehicle::new(start_pos, 0, grid.pos(3, 0), &mut grid).unwrap();

        let reservation = reserve(&mut grid, yield_to_pos).unwrap();

        vehicle.update(&mut grid);

        // vehicle should not move
        assert_eq!(vehicle.pos, start_pos);
        assert_eq!(vehicle.blocking_tile.unwrap(), yield_to_pos);

        drop(reservation)
    }

    #[test]
    fn test_yield_roundabout() {
        let mut grid = Grid::new_from_string(
            "\
            __.^__
            __.^__
            <<lr<<
            >>LR>>
            __.^__
            __.^__
            ",
        );

        let mut vehicle_top = Vehicle::new(grid.pos(2, 1), 0, grid.pos(2, 4), &mut grid).unwrap();

        let mut vehicle_left = Vehicle::new(grid.pos(1, 3), 1, grid.pos(5, 3), &mut grid).unwrap();

        let mut vehicle_bottom =
            Vehicle::new(grid.pos(3, 4), 2, grid.pos(3, 0), &mut grid).unwrap();

        let mut vehicle_right = Vehicle::new(grid.pos(4, 2), 3, grid.pos(0, 2), &mut grid).unwrap();

        assert!(vehicle_top.path.is_some());
        assert!(vehicle_left.path.is_some());
        assert!(vehicle_bottom.path.is_some());
        assert!(vehicle_right.path.is_some());

        println!("grid: \n{:?}", grid);

        vehicle_top.update(&mut grid);
        vehicle_left.update(&mut grid);
        vehicle_bottom.update(&mut grid);
        vehicle_right.update(&mut grid);

        println!("grid after: \n{:?}", grid);

        assert!(vehicle_top.blocking_tile.is_none());
        assert!(vehicle_left.blocking_tile.is_some());
        assert!(vehicle_bottom.blocking_tile.is_none());
        assert!(vehicle_right.blocking_tile.is_some());
    }

    #[test]
    fn test_yield_building() {
        // Houses should yield, but only to relevant traffic
        let mut grid = Grid::new_from_string(
            "\
            <<<<
            >>>>
            _h__",
        );

        let mut vehicle = Vehicle::new(grid.pos(1, 2), 0, grid.pos(3, 1), &mut grid).unwrap();

        let yield_to_pos = grid.pos(0, 1);

        assert!(vehicle.path.is_some());

        // reserve position we should yield to
        let reservation = reserve(&mut grid, yield_to_pos).unwrap();

        vehicle.update(&mut grid);

        assert_eq!(vehicle.blocking_tile.unwrap(), yield_to_pos);

        // grid.unreserve_position(&yield_to_pos);
        drop(reservation);

        // reserve position accross the street
        let do_not_yield_to_pos = grid.pos(1, 0);
        let reservation = reserve(&mut grid, do_not_yield_to_pos).unwrap();

        vehicle.update(&mut grid);

        assert_eq!(vehicle.blocking_tile, None);

        drop(reservation);
    }
}
