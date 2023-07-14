use std::collections::HashMap;

use ambient_api::prelude::*;
use flowerpot::CHUNK_SIZE;

use components::map::*;
use messages::{LoadChunk, OnPlayerLoadChunk, Ready};

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
    for x in -8..=8 {
        for y in -8..=8 {
            let position = IVec2::new(x, y);
            let chunk = Entity::new().with(chunk(), position).spawn();

            let tile_num = CHUNK_SIZE * CHUNK_SIZE;
            let mut chunk_tiles = Vec::with_capacity(tile_num);
            let mut tile_idx = 0;
            for ty in 0..CHUNK_SIZE {
                for tx in 0..CHUNK_SIZE {
                    let tile = Entity::new()
                        .with(in_chunk(), chunk)
                        .with(chunk_tile_index(), tile_idx)
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

    let all_chunks = query(chunk()).build();
    Ready::subscribe(move |source, _| {
        let player = source.client_entity_id().unwrap();
        let uid = source.client_user_id().unwrap();
        let chunks = all_chunks.evaluate();
        println!("Updating client {} with {} chunks", uid, chunks.len());
        for (e, position) in chunks {
            LoadChunk::new(position).send_client_targeted_reliable(uid.clone());
            OnPlayerLoadChunk::new(e, position, player, uid.clone()).send_local_broadcast(true);
        }
    });
}
