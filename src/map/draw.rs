use macroquad::{
    color::{Color, BLUE, RED, WHITE},
    math::Rect,
};

use crate::tileset::{Sprite, Tileset};

use super::{
    building::{Building, BuildingType, BUILDING_SIZE},
    city::City,
    grid::Grid,
    position::PIXEL_SIZE,
    tile::{Ramp, Road, Tile},
    vehicle::Vehicle,
    Direction, Map, Position,
};

const ROAD_INTERSECTION_SPRITE: Sprite = Sprite::new(3, 0);
const ROAD_ARROW_SPRITE: Sprite = Sprite::new(3, 1);
const ROAD_STRAIGHT_SPRITE: Sprite = Sprite::new(3, 2);
const ROAD_TURN_SPRITE: Sprite = Sprite::new(3, 3);
// const ROAD_YIELD_SPRITE: Sprite = Sprite::new(5, 2);
pub const ROAD_RAMP_SPRITE: Sprite = Sprite::new(3, 5);
pub const ROAD_RAMP_BASE_SPRITE: Sprite = Sprite::new(3, 6);
const ROAD_BRIDGE_SPRITE: Sprite = Sprite::new(3, 5);

const SHADOW_COLOR: Color = Color::new(0., 0., 0., 0.3);

const CAR_SPRITE: Sprite = Sprite::new(0, 1);
const CAR_SHADOW_SPRITE: Sprite = Sprite::new(0, 2);
const CAR_NO_PATH_SPRITE: Sprite = Sprite::new(12, 0);
const CAR_VERY_LATE_SPRITE: Sprite = Sprite::new(12, 1);

const WATER_SPRITE: Sprite = Sprite::new(4, 0);

const HOUSE_SPRITE: Sprite = Sprite::new_size(6, 0, BUILDING_SIZE);
const DRIVEWAY_SPRITE: Sprite = Sprite::new(4, 4);
const STATION_SPRITE: Sprite = Sprite::new_size(6, 2, BUILDING_SIZE);
const SPAWNER_SPRITE: Sprite = Sprite::new_size(6, 4, BUILDING_SIZE);
const PROGRESS_BOX: Sprite = Sprite::new_size(6, 6, BUILDING_SIZE);
// const PROGRESS_BAR: Sprite = Sprite::new_size(6, 8, BUILDING_SIZE);

// Shadow offset
pub const GRID_Z_OFFSET: f32 = 10.;

pub fn draw_map(map: &Map, tileset: &Tileset) {
    draw_grid_tiles(&map.grid, tileset);

    for b in map.grid.buildings.hash_map.values() {
        draw_building(b, tileset, &map.grid);
    }

    for s in map.vehicles.hash_map.iter() {
        if s.1.pos.grid_pos.z == 0 {
            draw_vehicle(s.1, tileset);
        }
    }

    draw_bridges(&map.grid, tileset);

    for s in map.vehicles.hash_map.iter() {
        if s.1.pos.grid_pos.z == 1 {
            draw_vehicle(s.1, tileset);
        }
    }

    for c in map.cities.hash_map.values() {
        draw_city(c, tileset);
    }
    if let Some(hint) = &map.metadata.level_hint {
        let pos: Position = (map.grid.size().0 / 2, map.grid.size().1 / 2 + 2).into();
        tileset.draw_text(hint.as_str(), 16., WHITE, &pos.into());
    }
}

pub fn draw_city(city: &City, tileset: &Tileset) {
    let mut rect: Rect = city.pos.round_to(2).into();
    rect.w *= 2.;
    rect.h *= 2.;
    tileset.draw_text(city.name.as_str(), 32., WHITE, &rect);
}

pub fn draw_grid_tiles(grid: &Grid, tileset: &Tileset) {
    let color: Color = Color::from_hex(0x2b313f);
    let mut rect: Rect = Position::new(0, 0).into();
    rect.w *= grid.tiles[0].len() as f32;
    rect.h *= grid.tiles.len() as f32;
    tileset.draw_rect(&rect, color);

    for (y, row) in grid.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            draw_tile(&tile.ground, (x as i16, y as i16).into(), tileset, grid);
        }
    }
}

pub fn draw_bridges(grid: &Grid, tileset: &Tileset) {
    for (y, row) in grid.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            draw_bridge(&tile.bridge, (x as i16, y as i16, 1).into(), tileset, grid);
        }
    }
}

pub fn draw_tile(tile: &Tile, pos: Position, tileset: &Tileset, grid: &Grid) {
    match tile {
        Tile::Road(road) => draw_road(road, pos, tileset, grid),
        Tile::Ramp(ramp) => draw_ramp(ramp, pos, tileset),
        Tile::Water => tileset.draw_tile(WATER_SPRITE, WHITE, &pos.into(), 0.),
        // Tile::Empty => tileset.draw_rect(&rect, LIGHTGRAY),
        _ => {}
    }
}

pub fn draw_bridge(tile: &Tile, pos: Position, tileset: &Tileset, grid: &Grid) {
    if let Tile::Road(road) = tile {
        draw_road_bridge(road, &pos, tileset, grid);
    }
}

fn draw_ramp(ramp: &Ramp, pos: Position, tileset: &Tileset) {
    let rect: &Rect = &pos.into();
    tileset.draw_tile(ROAD_RAMP_BASE_SPRITE, WHITE, rect, ramp.dir.to_radians());
    tileset.draw_tile(ROAD_RAMP_SPRITE, WHITE, rect, ramp.dir.to_radians());
}

pub fn draw_road(road: &Road, pos: Position, tileset: &Tileset, grid: &Grid) {
    let connection_count = road.connection_count();
    let rect: &Rect = &pos.into();

    if connection_count == 0 {
        tileset.draw_tile(
            ROAD_TURN_SPRITE,
            WHITE,
            rect,
            pos.default_connections()[0].to_radians(),
        );
    } else if connection_count != 1 {
        // draw intersection
        tileset.draw_tile(ROAD_INTERSECTION_SPRITE, WHITE, rect, 0.0);
        for dir in road.get_connections(&pos) {
            tileset.draw_tile(ROAD_ARROW_SPRITE, WHITE, rect, dir.to_radians());
        }
    } else {
        let dir = road.get_connections(&pos).first().unwrap().flatten();

        let connected_to: bool = match grid.get_tile(&(pos + dir.inverse())) {
            Some(Tile::Road(road)) => road.is_connected(dir),
            Some(Tile::Ramp(_)) => true,
            _ => false,
        };

        if connected_to {
            tileset.draw_tile(ROAD_STRAIGHT_SPRITE, WHITE, rect, dir.to_radians());
        } else {
            tileset.draw_tile(ROAD_TURN_SPRITE, WHITE, rect, dir.to_radians());
        }
    }

    // if self.reserved {
    //     tileset.draw_rect(&rect, RESERVED_PATH_COLOR);
    // }
}

pub fn draw_road_bridge(road: &Road, pos: &Position, tileset: &Tileset, grid: &Grid) {
    // shadow
    let ramp_below = matches!(
        grid.get_tile(&(*pos + Direction::LAYER_DOWN)),
        Some(Tile::Ramp(_))
    );
    let mut shadow_rect = Rect::from(*pos + Direction::LAYER_DOWN_2);
    shadow_rect.x += GRID_Z_OFFSET;
    if !ramp_below {
        tileset.draw_rect(&shadow_rect, SHADOW_COLOR);
    }

    let rect = Rect::from(*pos);
    for dir in road.get_connections(pos) {
        if ramp_below {
            //     if dir.z != 0 {
            //         let dir = dir.inverse();
            //         tileset.draw_tile(ROAD_RAMP_SPRITE, WHITE, &rect, dir.to_radians());
            //     } else {
            //         tileset.draw_tile(ROAD_RAMP_SPRITE, WHITE, &rect, dir.to_radians());
            //     }
        } else {
            tileset.draw_tile(ROAD_BRIDGE_SPRITE, WHITE, &rect, dir.to_radians());
        }
    }
}

pub fn draw_building(building: &Building, tileset: &Tileset, grid: &Grid) {
    let (sprite, color): (&Sprite, Color) = match building.building_type {
        BuildingType::House => (&HOUSE_SPRITE, WHITE),
        BuildingType::Station => (&STATION_SPRITE, WHITE),
        BuildingType::Spawner => (&SPAWNER_SPRITE, {
            let mut color = building.color.color();
            color.a *= 0.8;
            color
        }),
    };

    // draw connecting roads...
    if let Some((pos, dir)) = building.spawn_pos(grid) {
        tileset.draw_tile(DRIVEWAY_SPRITE, WHITE, &pos.into(), dir.to_radians());
    } else {
        // TODO: Draw some kind of Not-connected indicator
    }
    // draw connecting roads...
    if let Some((pos, dir)) = building.destination_pos(grid) {
        tileset.draw_tile(DRIVEWAY_SPRITE, WHITE, &pos.into(), dir.to_radians());
    }

    let rect = building.pos.into();

    if building.building_type == BuildingType::House {
        tileset.draw_tile(*sprite, color, &rect, 0.0);
    } else {
        tileset.draw_tile(PROGRESS_BOX, color, &rect, 0.0);

        let rect = Rect::new(
            rect.x + PIXEL_SIZE * 6. - 0.1,
            rect.y + PIXEL_SIZE * 8. - 0.1,
            PIXEL_SIZE * 2. * building.arrived_count.min(10) as f32 + 0.2,
            PIXEL_SIZE * 16. + 0.2,
        );

        tileset.draw_rect(&rect, color);
    }

    // for _ in 0..building.arrived_count.min(8) {
    //     // let progress_rect = rect * (GRID_CELL_SIZE.0 / 8. * i as f32);
    //     tileset.draw_tile(PROGRESS_BAR, WHITE, &rect, 0.0);
    //     rect.x += GRID_CELL_SIZE.0 as f32 / 6.;
    // }
    // // tileset.draw_text(
    //     format!("{}", building.arrived_count).as_str(),
    //     16.,
    //     WHITE,
    //     &rect,
    // );
}

pub fn draw_vehicle(vehicle: &Vehicle, tileset: &Tileset) {
    let mut rect = Rect::from(vehicle.pos.grid_pos);
    let dir = vehicle.pos.dir * vehicle.pos.lag_pos as i8;

    rect.x -= dir.x as f32;
    rect.y -= dir.y as f32; // - (self.lag_pos_pixels.z as f32) / (GRID_CELL_SIZE.0 / 10.);

    // let vehicle_red = Color::from_hex(0xf9524c);
    // let vehicle_blue = Color::from_hex(0xa0dae8);
    // let vehicle_yellow = Color::from_hex(0xf8c768);

    // let mut color = vehicle_blue;

    // draw shadow
    let mut shadow_rect = rect;
    shadow_rect.x += 2.;
    shadow_rect.y += 2.;
    let rotation = vehicle.pos.dir.to_radians();
    tileset.draw_tile(CAR_SHADOW_SPRITE, WHITE, &shadow_rect, rotation);

    tileset.draw_tile(CAR_SPRITE, vehicle.color.color(), &rect, rotation);

    if vehicle.path.grid_path.is_none() {
        tileset.draw_icon(CAR_NO_PATH_SPRITE, &rect, rotation);
    }
    else if vehicle.path.trip_late() < 0.75 {
        tileset.draw_icon(CAR_VERY_LATE_SPRITE, &rect, rotation);
        // tileset.draw_text("!", 32., colors::RED, &rect);
    }
}

pub fn draw_vehicle_detail(map: &Map, vehicle: &Vehicle, tileset: &Tileset) {
    // draw reserved
    let mut reserved_path_color = RED;
    reserved_path_color.a = 0.3;
    // for pos in self.reserved {
    //     tileset.draw_rect(&Rect::from(pos), reserved_path_color);
    // }

    let mut path_color = BLUE;
    path_color.a = 0.3;
    if let Some(path) = vehicle.path.grid_path.as_ref() {
        for pos in &path.0 {
            tileset.draw_rect(&Rect::from(*pos), path_color);
        }
    } else {
        let start_pos = vehicle.pos.grid_pos + vehicle.pos.dir;
        map.grid
            .iter_reachable(start_pos, |pos| tileset.draw_rect(&pos.into(), path_color));

        if let Some(building) = map.get_building(&vehicle.path.destination) {
            if let Some(end_pos) = building.destination_pos(&map.grid) {
                map.grid.iter_reachable(end_pos.0, |pos| tileset.draw_rect(&pos.into(), reserved_path_color));
            }

        }
    }
}
