use macroquad::{
    color::{Color, WHITE},
    math::Rect,
};

use crate::tileset::{Sprite, Tileset};

use super::{
    grid::{Grid, GRID_Z_OFFSET},
    tile::{Ramp, Road, Tile},
    Direction, Map, Position,
};

const ROAD_INTERSECTION_SPRITE: Sprite = Sprite::new(3, 0);
const ROAD_ARROW_SPRITE: Sprite = Sprite::new(3, 1);
const ROAD_STRAIGHT_SPRITE: Sprite = Sprite::new(3, 2);
const ROAD_TURN_SPRITE: Sprite = Sprite::new(3, 3);
// const ROAD_YIELD_SPRITE: Sprite = Sprite::new(5, 2);
pub const ROAD_RAMP_SPRITE: Sprite = Sprite::new_size(3, 5, (1, 1));
pub const ROAD_RAMP_BASE_SPRITE: Sprite = Sprite::new_size(3, 6, (1, 1));
const ROAD_BRIDGE_SPRITE: Sprite = Sprite::new(3, 5);

const SHADOW_COLOR: Color = Color::new(0., 0., 0., 0.3);

pub fn draw_map(map: &Map, tileset: &Tileset) {
    draw_grid_tiles(&map.grid, tileset);

    for b in map.grid.buildings.hash_map.values() {
        b.draw(tileset);
    }

    for s in map.vehicles.hash_map.iter() {
        if s.1.pos.z == 0 {
            s.1.draw(tileset);
        }
    }

    draw_bridges(&map.grid, tileset);

    for s in map.vehicles.hash_map.iter() {
        if s.1.pos.z == 1 {
            s.1.draw(tileset);
        }
    }

    for c in map.cities.hash_map.values() {
        c.draw(tileset);
    }
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
        Tile::Road(road) => draw_road(&road, pos, tileset, grid),
        Tile::Ramp(ramp) => draw_ramp(&ramp, pos, tileset),
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
        for dir in road.iter_connections(&pos) {
            tileset.draw_tile(ROAD_ARROW_SPRITE, WHITE, rect, dir.to_radians());
        }
    } else {
        let dir = road.iter_connections(&pos).first().unwrap().flatten();

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
    for dir in road.iter_connections(pos) {
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
