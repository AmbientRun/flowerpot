use std::collections::HashMap;

use ambient_api::{core::network::components::no_sync, prelude::*};
use flowerpot_common::CHUNK_SIZE;

use packages::{
    region_networking::components::{in_region, players_observing},
    this::components::*,
};

mod shared;

pub fn stitch_neighbors(entities: HashMap<IVec2, EntityId>) {
    for (pos, e) in entities.iter() {
        let pos = *pos;
        let e = *e;

        if let Some(n) = entities.get(&(pos + IVec2::X)).copied() {
            entity::add_component(e, east_neighbor(), n);
            entity::add_component(n, west_neighbor(), e);
        }

        if let Some(n) = entities.get(&(pos + IVec2::Y)).copied() {
            entity::add_component(e, south_neighbor(), n);
            entity::add_component(n, north_neighbor(), e);
        }
    }
}

#[main]
pub fn main() {
    shared::init_shared_map();

    let mut chunks = HashMap::new();
    let mut tiles = HashMap::new();
    for x in -4..=4 {
        for y in -4..=4 {
            let position = IVec2::new(x, y);
            let chunk = Entity::new()
                .with(chunk(), position)
                .with(no_sync(), ())
                .with(players_observing(), vec![])
                .spawn();

            let tile_num = CHUNK_SIZE * CHUNK_SIZE;
            let mut chunk_tiles = Vec::with_capacity(tile_num);
            let mut tile_idx = 0;
            for ty in 0..CHUNK_SIZE {
                for tx in 0..CHUNK_SIZE {
                    let tile = Entity::new()
                        .with(in_chunk(), chunk)
                        .with(chunk_tile_index(), tile_idx)
                        .with(no_sync(), ())
                        .spawn();

                    chunk_tiles.push(tile);
                    let position = position * CHUNK_SIZE as i32 + ivec2(tx as i32, ty as i32);
                    tiles.insert(position, tile);
                    tile_idx += 1;
                }
            }

            entity::add_component(chunk, chunk_tile_refs(), chunk_tiles);

            chunks.insert(position, chunk);
        }
    }

    // TODO faster stitching inside of the loop
    stitch_neighbors(chunks);
    stitch_neighbors(tiles);

    spawn_query(in_chunk()).bind(move |entities| {
        for (e, chunk) in entities {
            entity::add_component(e, in_region(), chunk);
        }
    });

    change_query(in_chunk())
        .track_change(in_chunk())
        .bind(move |entities| {
            for (e, chunk) in entities {
                entity::add_component(e, in_region(), chunk);
            }
        });

    entity::add_component(entity::resources(), is_mod_loaded(), ());
}
