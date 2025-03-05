use macroquad::{
    color::{Color, BLUE, RED, WHITE},
    math::Rect,
};

use crate::{
    consts::SpawnerColors,
    tileset::{Sprite, Tileset},
};

use super::{
    building::{Building, BuildingType, BUILDING_SIZE},
    city::City,
    grid::Grid,
    position::PIXEL_SIZE,
    route::Route,
    tile::{Road, Tile},
    vehicle::Vehicle,
    Direction, Map, Position, Tick,
};

const ROAD_INTERSECTION_SPRITE: Sprite = Sprite::new_size(2, 0, BUILDING_SIZE);
const ROAD_TWO_WAY_SPRITE: Sprite = Sprite::new_size(2, 2, BUILDING_SIZE);
const ROAD_TWO_WAY_TURN_SPRITE: Sprite = Sprite::new_size(2, 4, BUILDING_SIZE);
const ROAD_DEAD_END_SPRITE: Sprite = Sprite::new_size(2, 6, BUILDING_SIZE);
const ROAD_NOT_CONNECTED: Sprite = Sprite::new_size(2, 8, BUILDING_SIZE);
const _ROAD_ONE_WAY_SPRITE: Sprite = Sprite::new_size(2, 10, BUILDING_SIZE);
const _ROAD_ONE_WAY_TURN_SPRITE: Sprite = Sprite::new_size(2, 12, BUILDING_SIZE);
const ROAD_T_INTERSECTION_SPRITE: Sprite = Sprite::new_size(2, 14, BUILDING_SIZE);
const ROAD_TWO_WAY_BRIDGE_SPRITE: Sprite = Sprite::new_size(4, 12, BUILDING_SIZE);
const _ROAD_ONE_WAY_BRIDGE_SPRITE: Sprite = Sprite::new_size(4, 14, BUILDING_SIZE);

const SHADOW_COLOR: Color = Color::new(0., 0., 0., 0.3);

const CAR_SPRITE: Sprite = Sprite::new(0, 1);
const CAR_SHADOW_SPRITE: Sprite = Sprite::new(1, 1);
const BUS_SPRITE: Sprite = Sprite::new(0, 2);
const BUS_SHADOW_SPRITE: Sprite = Sprite::new(1, 2);
const CAR_NO_PATH_SPRITE: Sprite = Sprite::new(12, 0);
const CAR_VERY_LATE_SPRITE: Sprite = Sprite::new(12, 1);

const WATER_SPRITE: Sprite = Sprite::new(4, 0);
const MOUNTAIN_SPRITE: Sprite = Sprite::new(4, 2);

const HOUSE_SPRITE: Sprite = Sprite::new_size(6, 0, BUILDING_SIZE);
const DRIVEWAY_SPRITE: Sprite = Sprite::new(4, 4);
const STATION_SPRITE: Sprite = Sprite::new_size(6, 2, BUILDING_SIZE);
const SPAWNER_SPRITE: Sprite = Sprite::new_size(6, 4, BUILDING_SIZE);
const PROGRESS_BOX: Sprite = Sprite::new_size(6, 6, BUILDING_SIZE);
// const PROGRESS_BAR: Sprite = Sprite::new_size(6, 8, BUILDING_SIZE);

pub fn draw_map(map: &Map, tileset: &Tileset) {
    draw_grid_tiles(&map.grid, tileset);

    for b in map.grid.buildings.hash_map.values() {
        draw_building(b, tileset, &map.grid);
    }

    // TODO: Draw before buildings but after driveways!
    for s in map.vehicles.hash_map.values() {
        if s.pos.pos.z == 0 {
            draw_vehicle(s, tileset, map.tick);
        }
    }

    draw_bridges(&map.grid, tileset);

    for s in map.vehicles.hash_map.values() {
        if s.pos.pos.z == 1 || s.dir.z as i16 == -1 {
            draw_vehicle(s, tileset, map.tick);
        }
    }

    for (index, r) in map.routes.iter().enumerate() {
        draw_route(r, index, map, tileset);
    }

    for c in map.cities.hash_map.values() {
        draw_city(c, tileset);
    }
    if let Some(hint) = &map.metadata.level_hint {
        let pos: Position = Position::new(0, 0) + map.grid.size() / 2 + (Direction::DOWN * 2);
        tileset.draw_text(hint.as_str(), 16., WHITE, &pos.into());
    }
}

pub fn draw_route(r: &Route, index: usize, map: &Map, tileset: &Tileset) {
    // This should be changed to a path...
    let mut prev_pos: Option<Rect> = None;
    for s in &r.stops {
        if let Some(building) = map.get_building(s) {
            let color = SpawnerColors::from_number(index as u64).color();
            let rect: Rect = building.pos.into();
            if let Some(previous) = prev_pos {
                tileset.draw_line(&previous, &rect, 10., color);
            }
            prev_pos = Some(rect);
            tileset.draw_circle(&rect, 20., color);
        }
    }
}

pub fn draw_city(city: &City, tileset: &Tileset) {
    let mut rect: Rect = city.pos.round_to(2).into();
    rect.w *= 2.;
    rect.h *= 2.;
    tileset.draw_text(
        format!(
            "{} {}/{}",
            city.name,
            city.houses_goal_met,
            city.houses.len()
        )
        .as_str(),
        32.,
        WHITE,
        &rect,
    );
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
            draw_bridge(&tile.bridge, (x as i16, y as i16, 1).into(), tileset);
        }
    }
}

pub fn draw_tile(tile: &Tile, pos: Position, tileset: &Tileset, grid: &Grid) {
    match tile {
        Tile::Road(road) => draw_road(road, pos, tileset, grid),
        Tile::Water => tileset.draw_tile(WATER_SPRITE, WHITE, &pos.into(), 0.),
        Tile::Mountain => tileset.draw_tile(MOUNTAIN_SPRITE, WHITE, &pos.into(), 0.),
        // Tile::Empty => tileset.draw_rect(&rect, LIGHTGRAY),
        _ => {}
    }
}

pub fn draw_bridge(tile: &Tile, pos: Position, tileset: &Tileset) {
    if let Tile::Road(road) = tile {
        draw_road_bridge(road, &pos, tileset);
    }
}

pub fn draw_road(road: &Road, pos: Position, tileset: &Tileset, grid: &Grid) {
    let rect: &Rect = &pos.into();

    let mut connections: Vec<Direction> = road.connections.iter().collect();

    for dir in Direction::ALL {
        if !road.connections.is_connected_flat(dir) {
            if let Some(Tile::Road(road)) = grid.get_tile(&(pos + dir)) {
                if road.connections.is_connected_flat(dir.inverse()) {
                    connections.push(dir);
                }
            }
        }
    }

    let mut first_dir = connections.first().unwrap_or(&Direction::NONE);

    let connection_count = connections.len();

    if connection_count == 0 {
        tileset.draw_tile(ROAD_NOT_CONNECTED, WHITE, rect, 0.0);
    } else if connection_count == 4 {
        // draw intersection
        tileset.draw_tile(ROAD_INTERSECTION_SPRITE, WHITE, rect, 0.0);
        // for dir in road.get_connections() {
        // tileset.draw_tile(ROAD_ARROW_SPRITE, WHITE, rect, dir.to_radians());
        // }
    } else if connection_count == 3 {
        // find the one connection we aren't
        for dir in &Direction::ALL {
            if !connections.iter().any(|con_dir| con_dir.flatten() == *dir) {
                first_dir = dir;
            }
        }
        tileset.draw_tile(
            ROAD_T_INTERSECTION_SPRITE,
            WHITE,
            rect,
            first_dir.to_radians(),
        );
    } else if connection_count == 2 {
        if connections[0].flatten().inverse() == connections[1].flatten() {
            tileset.draw_tile(
                if first_dir.f {
                    _ROAD_ONE_WAY_SPRITE
                } else {
                    ROAD_TWO_WAY_SPRITE
                },
                WHITE,
                rect,
                first_dir.to_radians(),
            );
        } else {
            let mut flip = false;
            let turn_dir: Direction =
                if connections[0].rotate_left().flatten() != connections[1].flatten() {
                    connections[1]
                } else {
                    flip = true;
                    connections[0].rotate_left()
                };
            tileset.draw_tile_ex(
                if first_dir.f {
                    _ROAD_ONE_WAY_TURN_SPRITE
                } else {
                    ROAD_TWO_WAY_TURN_SPRITE
                },
                WHITE,
                rect,
                turn_dir.to_radians(),
                flip,
            );
        }
    } else
    /* if connection_count == 1 */
    {
        if first_dir.f {
            // TODO: Road one-way dead end? (this requires fixing the above logic so we actually get here...)
            // todo!();
            tileset.draw_tile(_ROAD_ONE_WAY_SPRITE, WHITE, rect, first_dir.to_radians());
        } else {
            tileset.draw_tile(ROAD_DEAD_END_SPRITE, WHITE, rect, first_dir.to_radians());
        }
    }
}

pub fn draw_road_bridge(road: &Road, pos: &Position, tileset: &Tileset) {
    let shadow_rect = Rect::from(*pos + Direction::LAYER_DOWN_2);
    // shadow_rect.x += GRID_Z_OFFSET;
    tileset.draw_rect(&shadow_rect, SHADOW_COLOR);

    let rect = Rect::from(*pos);
    for dir in road.connections.iter() {
        tileset.draw_tile(
            ROAD_TWO_WAY_BRIDGE_SPRITE,
            WHITE,
            &rect,
            dir.rotate_left().to_radians(),
        );
    }
}

pub fn draw_building(building: &Building, tileset: &Tileset, grid: &Grid) {
    let (sprite, color): (&Sprite, Color) = match building.building_type {
        BuildingType::House => (
            &HOUSE_SPRITE,
            if building.arrived_goal_met() {
                WHITE
            } else {
                RED
            },
        ),
        BuildingType::Station => (&STATION_SPRITE, WHITE),
        BuildingType::Spawner => (&SPAWNER_SPRITE, {
            let mut color = building.color.color();
            color.a *= 0.8;
            color
        }),
    };

    let rect: Rect = building.pos.into();

    if matches!(building.building_type, BuildingType::House) {
        // draw driveway
        if let Some((_pos, dir)) = building.destination_pos(grid) {
            let mut rect = rect;
            // Push the driveway over so it lines up with the road
            rect.x += dir.x as f32 * PIXEL_SIZE;
            rect.y += dir.y as f32 * PIXEL_SIZE;
            tileset.draw_tile(DRIVEWAY_SPRITE, WHITE, &rect, dir.to_radians());
        }
        // TODO: Draw some kind of not-connected indicator
    }

    if building.building_type == BuildingType::House {
        tileset.draw_tile(*sprite, color, &rect, 0.0);
    } else {
        tileset.draw_tile(PROGRESS_BOX, color, &rect, 0.0);

        let progrewss: f32 = if matches!(building.building_type, BuildingType::Spawner) {
            building.arrived_count.min(10) as f32
        } else {
            building.cargo_count.min(10) as f32
        };

        let rect = Rect::new(
            rect.x + PIXEL_SIZE * 3. - 0.1,
            rect.y + PIXEL_SIZE * 4. - 0.1,
            PIXEL_SIZE * 1. * progrewss + 0.2,
            PIXEL_SIZE * 8. + 0.2,
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

pub fn draw_vehicle(vehicle: &Vehicle, tileset: &Tileset, _tick: Tick) {
    let mut rect = Rect::from(vehicle.pos);
    let dir = vehicle.dir * vehicle.lag_pos as i8;

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
    let rotation = vehicle.dir.to_radians();

    let (sprite, shadow_sprite) = if vehicle.route_info.is_none() {
        (CAR_SPRITE, CAR_SHADOW_SPRITE)
    } else {
        (BUS_SPRITE, BUS_SHADOW_SPRITE)
    };

    tileset.draw_tile(shadow_sprite, WHITE, &shadow_rect, rotation);

    tileset.draw_tile(sprite, vehicle.color.color(), &rect, rotation);

    if vehicle.grid_path.is_none() {
        tileset.draw_icon(CAR_NO_PATH_SPRITE, &rect, rotation);
    } else if vehicle.trip_late() < 0.25 {
        tileset.draw_icon(CAR_VERY_LATE_SPRITE, &rect, rotation);
        // tileset.draw_text("!", 32., colors::RED, &rect);
    }
}

pub fn draw_vehicle_detail(map: &Map, vehicle: &Vehicle, tileset: &Tileset) {
    // draw reserved
    let mut reserved_path_color = RED;
    reserved_path_color.a = 0.3;

    for  res in vehicle.reserved.iter() {
        // let start = res.start.saturating_sub(map.tick);
        // let end: String = if res.end != Tick::MAX {
        //     format!("{}", res.end.saturating_sub(map.tick))
        // } else {
        //     "M".into()
        // };
        // tileset.draw_text(
        //     format!("{i}:{start}-{end}").as_str(),
        //     12.,
        //     WHITE,
        //     &res.pos.into(),
        // )
        tileset.draw_rect(&res.pos.pos.into(), reserved_path_color);
    }

    let mut path_color = BLUE;
    path_color.a = 0.3;
    if let Some(path) = vehicle.grid_path.as_ref() {
        for pos in &path.0 {
            tileset.draw_rect(&Rect::from(*pos), path_color);
        }
    } else {
        let start_pos = vehicle.pos + vehicle.dir;
        map.grid.iter_reachable(start_pos.into(), |pos| {
            tileset.draw_rect(&pos.into(), path_color)
        });

        if let Some(building) = map.get_building(&vehicle.destination) {
            if let Some(end_pos) = building.destination_pos(&map.grid) {
                map.grid.iter_reachable(end_pos.0, |pos| {
                    tileset.draw_rect(&pos.into(), reserved_path_color)
                });
            }
        }
    }
}
