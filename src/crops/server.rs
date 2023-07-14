use std::collections::HashMap;

use ambient_api::prelude::*;

use components::{
    crops::*,
    map::{chunk, chunk_tile_index, chunk_tile_refs, in_chunk},
};

use messages::{OnPlayerLoadChunk, UpdateMediumCrops};

mod shared;

#[main]
fn main() {
    let dummy_class = Entity::new().with_default(medium_crop_class()).spawn();

    spawn_query((chunk(), chunk_tile_refs())).bind(move |entities| {
        for (_, (chunk, tiles)) in entities {
            if chunk != IVec2::ZERO {
                continue;
            }

            let dummy_crop = Entity::new().with(class(), dummy_class).spawn();

            let tile = tiles[0];
            entity::add_component(tile, medium_crop_occupant(), dummy_crop);
        }
    });

    OnPlayerLoadChunk::subscribe(move |_, data| {
        let Some(chunk_tiles) = entity::get_component(data.chunk_entity, chunk_tile_refs()) else { return};

        let mut tiles = Vec::with_capacity(chunk_tiles.len());
        let mut classes = Vec::with_capacity(chunk_tiles.len());
        let mut dirty = false;
        for (tile_idx, tile) in chunk_tiles.iter().enumerate() {
            let occupant = match entity::get_component(*tile, medium_crop_occupant()) {
                Some(occupant) if !occupant.is_null() => occupant,
                _ => continue,
            };

            let Some(class) = entity::get_component(occupant, class()) else {
                    eprintln!("crop {} has no class", occupant);
                    continue;
                };

            dirty = true;
            tiles.push(tile_idx.try_into().unwrap());
            classes.push(class);
        }

        if !dirty {
            return;
        }

        UpdateMediumCrops::new(data.chunk_pos, classes, tiles)
            .send_client_targeted_reliable(data.player_uid.clone());
    });

    // TODO subscription-based batch updates
    change_query((in_chunk(), chunk_tile_index(), medium_crop_occupant()))
        .track_change(medium_crop_occupant())
        .bind(move |entities| {
            type ChunkUpdate = Vec<(u8, EntityId)>;
            type BatchedUpdates = HashMap<IVec2, ChunkUpdate>;
            let mut updates = BatchedUpdates::new();

            for (_e, (chunk_ref, tile_idx, occupant)) in entities {
                let Some(chunk_pos) = entity::get_component(chunk_ref, chunk()) else { continue };

                let Some(class) = entity::get_component(occupant, class()) else {
                    eprintln!("crop {} has no class", occupant);
                    continue;
                };

                let update = (tile_idx, class);

                if let Some(updates) = updates.get_mut(&chunk_pos) {
                    updates.push(update);
                } else {
                    updates.insert(chunk_pos, vec![update]);
                }
            }

            for (chunk, update) in updates {
                let (tiles, classes): (Vec<_>, Vec<_>) = update.into_iter().unzip();
                UpdateMediumCrops::new(chunk, classes, tiles).send_client_broadcast_reliable();
            }
        });
}
