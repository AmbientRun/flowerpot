use std::collections::HashMap;

use ambient_api::prelude::*;
use flowerpot::CHUNK_SIZE;

use components::map::*;
use messages::{LoadChunk, OnPlayerLoadChunk, OnPlayerUnloadChunk, UnloadChunk};

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
            let chunk = Entity::new()
                .with(chunk(), position)
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

    OnPlayerLoadChunk::subscribe(move |_, data| {
        entity::mutate_component(data.chunk_entity, players_observing(), |observing| {
            let mut added = false;
            for (idx, p) in observing.iter().enumerate() {
                if *p > data.player_entity {
                    observing.insert(idx, data.player_entity);
                    added = true;
                    break;
                } else if *p == data.player_entity {
                    return;
                }
            }

            if !added {
                observing.push(data.player_entity);
            }

            LoadChunk::new(data.chunk_pos).send_client_targeted_reliable(data.player_uid);
        });
    });

    change_query((chunk(), players_observing()))
        .track_change(players_observing())
        .bind(move |entities| {
            for (_e, (chunk_pos, observing)) in entities {
                let is_sorted = observing.windows(2).all(|w| w[0] < w[1]);
                if !is_sorted {
                    eprintln!(
                        "list of observers to chunk {} isn't sorted: {:?}",
                        chunk_pos, observing
                    );
                }
            }
        });

    OnPlayerUnloadChunk::subscribe(move |_, data| {
        entity::mutate_component(data.chunk_entity, players_observing(), |observing| {
            let old_len = observing.len();
            observing.retain(|p| *p != data.player_entity);
            if observing.len() < old_len {
                UnloadChunk::new(data.chunk_pos).send_client_targeted_reliable(data.player_uid);
            }
        });
    });

    entity::add_component(entity::resources(), mod_loaded(), ());
}
