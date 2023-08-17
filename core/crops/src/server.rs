use std::collections::HashMap;

use ambient_api::{core::player::components::user_id, prelude::*};

use packages::{
    crops::{components::*, messages::*},
    map::{components::*, messages::OnPlayerLoadChunk},
};

mod shared;

#[main]
fn main() {
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

    change_query((in_chunk(), chunk_tile_index(), medium_crop_occupant()))
        .track_change(medium_crop_occupant())
        .bind(move |entities| {
            type ChunkUpdate = Vec<(u8, EntityId)>;
            type BatchedUpdates = HashMap<EntityId, ChunkUpdate>;
            let mut updates = BatchedUpdates::new();

            for (_e, (chunk_ref, tile_idx, occupant)) in entities {
                let class = if occupant.is_null() { EntityId::null()} else {
                let Some(class) = entity::get_component(occupant, class()) else {
                    eprintln!("crop {} has no class", occupant);
                    continue;
                };

                    class
                };

                let update = (tile_idx, class);

                if let Some(updates) = updates.get_mut(&chunk_ref) {
                    updates.push(update);
                } else {
                    updates.insert(chunk_ref, vec![update]);
                }
            }

            for (chunk_ref, update) in updates {
                let Some(chunk_pos) = entity::get_component(chunk_ref, chunk()) else { continue };
                let Some(observers) = entity::get_component(chunk_ref, players_observing()) else { continue };
                let (tiles, classes): (Vec<_>, Vec<_>) = update.into_iter().unzip();

                for observer in observers {
                    let Some(uid) = entity::get_component(observer, user_id()) else { continue };
                    UpdateMediumCrops::new(chunk_pos, classes.clone(), tiles.clone()).send_client_targeted_reliable(uid);
                }
            }
        });

    spawn_query((is_medium_crop(), class(), on_tile())).bind(move |entities| {
        for (e, (_medium, crop_class, tile)) in entities {
            if let Some(old_occupant) = entity::get_component(tile, medium_crop_occupant()) {
                if !old_occupant.is_null() && old_occupant != e {
                    entity::despawn_recursive(old_occupant);
                }
            }

            entity::add_components(
                e,
                entity::get_all_components(crop_class)
                    .with(age(), 0)
                    .with(class(), crop_class)
                    .with(on_tile(), tile),
            );

            entity::add_component(tile, medium_crop_occupant(), e);
        }
    });

    despawn_query((is_medium_crop(), class(), on_tile())).bind(move |entities| {
        for (e, (_medium, _class, tile)) in entities {
            if entity::get_component(tile, medium_crop_occupant()) == Some(e) {
                entity::set_component(tile, medium_crop_occupant(), EntityId::null());
            }
        }
    });

    run_async(async move {
        let all_crops = query((is_medium_crop(), on_tile(), age())).build();
        loop {
            sleep(0.1).await;

            for (e, (_medium, _tile, old_age)) in all_crops.evaluate() {
                let new_age = old_age + 1;
                entity::set_component(e, age(), new_age);
            }
        }
    });

    change_query((is_medium_crop(), on_tile(), age(), seeding_interval(), seed()))
        .track_change(age())
        .bind(move |entities| {
            for (_e, (_, tile, age, interval, seed)) in entities {
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
                        .with(is_medium_crop(), ())
                        .with(class(), seed)
                        .with(on_tile(), neighbor)
                        .spawn();

                    break;
                }
            }
        });

    change_query((
        is_medium_crop(),
        on_tile(),
        age(),
        next_growth_age(),
        next_growth_stage(),
    ))
    .track_change(age())
    .bind(move |entities| {
        for (e, (_, tile, current_age, next_age, next)) in entities {
            if current_age < next_age {
                continue;
            }

            entity::despawn_recursive(e);

            if !next.is_null() {
                Entity::new()
                    .with(is_medium_crop(), ())
                    .with(class(), next)
                    .with(on_tile(), tile)
                    .spawn();
            }
        }
    });
}
