use std::collections::HashMap;

use ambient_api::{components::core::player::user_id, prelude::*};

use components::{
    crops::*,
    map::{
        chunk, chunk_tile_index, east_neighbor, in_chunk, north_neighbor, south_neighbor,
        west_neighbor,
    },
};

use flowerpot::CHUNK_SIZE;
use messages::{OnPlayerLoadChunk, UpdateMediumCrops};

use crate::components::map::players_observing;

mod shared;

#[main]
fn main() {
    OnPlayerLoadChunk::subscribe(move |_, data| {
        let Some(occupants) = entity::get_component(data.chunk_entity, medium_crop_occupants()) else { return };

        let mut tiles = Vec::with_capacity(occupants.len());
        let mut classes = Vec::with_capacity(occupants.len());
        let mut dirty = false;
        for (tile_idx, occupant) in occupants.iter().enumerate() {
            if occupant.is_null() {
                continue;
            }

            let Some(class) = entity::get_component(*occupant, class()) else {
                    eprintln!("crop {} has no class", occupant);
                    continue;
                };

            eprintln!("loading class {} at {}", class, tile_idx);
            dirty = true;
            tiles.push(tile_idx.try_into().unwrap());
            classes.push(class);
        }

        if !dirty {
            return;
        }

        eprintln!("tiles updated: {:?}", tiles);

        UpdateMediumCrops::new(data.chunk_pos, classes, tiles)
            .send_client_targeted_reliable(data.player_uid.clone());
    });

    spawn_query(chunk()).bind(move |entities| {
        for (e, _) in entities {
            let tile_num = CHUNK_SIZE * CHUNK_SIZE;
            let occupants = vec![EntityId::null(); tile_num];
            entity::add_component(e, medium_crop_occupants(), occupants.clone());
            entity::add_component(e, last_medium_crop_occupants(), occupants.clone());
        }
    });

    change_query((
        chunk(),
        medium_crop_occupants(),
        last_medium_crop_occupants(),
        players_observing(),
    ))
    .track_change(medium_crop_occupants())
    .bind(move |entities| {
        for (e, (chunk_pos, occupants, last, observers)) in entities {
            entity::set_component(e, last_medium_crop_occupants(), occupants.clone());

            let mut changed = Vec::<(u8, EntityId)>::new();
            for (tile_idx, (new, old)) in occupants.iter().zip(last.iter()).enumerate() {
                if new != old {
                    let Some(new) = entity::get_component(*new, class()) else { continue };
                    let Some(old) = entity::get_component(*old, class()) else { continue };

                    if new != old {
                        changed.push((tile_idx.try_into().unwrap(), new));
                    }
                }
            }

            if changed.is_empty() {
                continue;
            }

            eprintln!("{:#?}", changed);

            let (tiles, classes): (Vec<_>, Vec<_>) = changed.into_iter().unzip();
            for observer in observers {
                let Some(uid) = entity::get_component(observer, user_id()) else { continue };
                UpdateMediumCrops::new(chunk_pos, classes.clone(), tiles.clone())
                    .send_client_targeted_reliable(uid);
            }
        }
    });

    spawn_query((medium_crop(), class(), in_chunk(), chunk_tile_index())).bind(move |entities| {
        type ChunkUpdate = Vec<(u8, EntityId)>;
        type BatchedUpdates = HashMap<EntityId, ChunkUpdate>;
        let mut updates = BatchedUpdates::new();

        for (e, (_medium, crop_class, chunk_ref, tile_idx)) in entities {
            entity::add_components(
                e,
                entity::get_all_components(crop_class)
                    .with(age(), 0)
                    .with_merge(entity::get_all_components(e)),
            );

            let update = (tile_idx, e);
            if let Some(updates) = updates.get_mut(&chunk_ref) {
                updates.push(update);
            } else {
                updates.insert(chunk_ref, vec![update]);
            }
        }

        for (chunk_ref, update) in updates {
            eprintln!("update: {:#?}", update);
            entity::mutate_component(chunk_ref, medium_crop_occupants(), |occupants| {
                for (tile_idx, new_crop) in update {
                    occupants[tile_idx as usize] = new_crop;
                }
            });
        }
    });

    despawn_query((medium_crop(), class(), in_chunk(), chunk_tile_index())).bind(move |entities| {
        type ChunkUpdate = Vec<(u8, EntityId)>;
        type BatchedUpdates = HashMap<EntityId, ChunkUpdate>;
        let mut updates = BatchedUpdates::new();

        for (e, (_medium, _crop_class, chunk_ref, tile_idx)) in entities {
            let update = (tile_idx, e);
            if let Some(updates) = updates.get_mut(&chunk_ref) {
                updates.push(update);
            } else {
                updates.insert(chunk_ref, vec![update]);
            }
        }

        for (chunk_ref, update) in updates {
            entity::mutate_component(chunk_ref, medium_crop_occupants(), |occupants| {
                for (tile_idx, despawned) in update {
                    let occupant = &mut occupants[tile_idx as usize];
                    if *occupant == despawned {
                        *occupant = EntityId::null();
                    }
                }
            });
        }
    });

    run_async(async move {
        let all_crops = query((medium_crop(), in_chunk(), age())).build();
        loop {
            sleep(0.1).await;

            for (e, (_medium, _chunk_ref, old_age)) in all_crops.evaluate() {
                let new_age = old_age + 1;
                entity::set_component(e, age(), new_age);
            }
        }
    });

    /*change_query((
        medium_crop(),
        in_chunk(),
        chunk_tile_index(),
        age(),
        seeding_interval(),
        seed(),
    ))
    .track_change(age())
    .bind(move |entities| {
        for (_e, (_, chunk_ref, tile_idx, age, interval, seed)) in entities {
            if age == 0 || age % interval != 0 {
                continue;
            }

            let mut neighbors = [
                north_neighbor(),
                east_neighbor(),
                south_neighbor(),
                west_neighbor(),
            ];

            let mut rng = thread_rng();
            neighbors.shuffle(&mut rng);

            for neighbor in neighbors {
                let Some(neighbor) = entity::get_component(tile, neighbor) else {  continue };

                if !entity::get_component(neighbor, medium_crop_occupant())
                    .unwrap_or_default()
                    .is_null()
                {
                    continue;
                }

                Entity::new()
                    .with_default(medium_crop())
                    .with(class(), seed)
                    .with(on_tile(), neighbor)
                    .spawn();

                break;
            }
        }
    });*/

    change_query((
        medium_crop(),
        in_chunk(),
        chunk_tile_index(),
        age(),
        next_growth_age(),
        next_growth_stage(),
    ))
    .track_change(age())
    .bind(move |entities| {
        for (e, (_, chunk_ref, tile_idx, current_age, next_age, next)) in entities {
            if current_age < next_age {
                continue;
            }

            entity::despawn_recursive(e);

            if !next.is_null() {
                Entity::new()
                    .with_default(medium_crop())
                    .with(in_chunk(), chunk_ref)
                    .with(chunk_tile_index(), tile_idx)
                    .with(class(), next)
                    .spawn();
            }
        }
    });
}
