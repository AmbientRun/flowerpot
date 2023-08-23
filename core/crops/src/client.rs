use ambient_api::{
    core::{
        prefab::components::{prefab_from_url, spawned},
        transform::{
            components::{local_to_parent, local_to_world, rotation, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use flowerpot_common::CHUNK_SIZE;
use packages::{
    crops::{components::*, messages::*},
    map::components::{chunk, chunk_tile_index, chunk_tile_refs, in_chunk, position},
    terrain::components::altitude,
};

mod shared;

#[main]
fn main() {
    let chunks = flowerpot_common::init_map(chunk());

    UpdateMediumCrops::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            let chunks = chunks.clone();
            run_async(async move {
                let mut tries = std::iter::repeat(()).take(20);
                let chunk = loop {
                    if let Some(_) = tries.next() {
                        if let Some(chunk) = chunks.lock().unwrap().get(&data.chunk).copied() {
                            break Some(chunk);
                        }
                    } else {
                        break None;
                    }

                    sleep(0.1).await;
                };

                let Some(chunk) = chunk else { return };
                let Some(tiles) = entity::get_component(chunk, chunk_tile_refs()) else {
                    return;
                };

                for (tile_idx, class) in data
                    .crop_tiles
                    .into_iter()
                    .zip(data.crop_classes.into_iter())
                {
                    let tile = tiles[tile_idx as usize];

                    let old_occupant =
                        entity::get_component(tile, medium_crop_occupant()).unwrap_or_default();

                    if class.is_null() {
                        if !old_occupant.is_null() {
                            entity::despawn_recursive(old_occupant);
                        }

                        continue;
                    }

                    let occupant_position = (data.chunk * CHUNK_SIZE as i32).as_vec2()
                        + vec2(
                            (tile_idx as usize % CHUNK_SIZE) as f32,
                            (tile_idx as usize / CHUNK_SIZE) as f32,
                        )
                        + 0.5;

                    let new_occupant = entity::get_all_components(class)
                        .with(is_medium_crop(), ())
                        .with(position(), occupant_position)
                        .with(in_chunk(), chunk)
                        .with(chunk_tile_index(), tile_idx)
                        .with(despawn_when_loaded(), old_occupant)
                        .spawn();

                    entity::add_component(tile, medium_crop_occupant(), new_occupant);
                }
            });
        }
    });

    // despawn old crops once the new one has finished loading
    spawn_query((despawn_when_loaded(), spawned())).bind(move |entities| {
        for (e, (old, _)) in entities {
            if entity::exists(old) {
                entity::remove_component(e, despawn_when_loaded());
                entity::despawn_recursive(old);
            }
        }
    });

    // despawn old crops if the new crop hasn't finished loading
    despawn_query(despawn_when_loaded()).bind(move |entities| {
        for (_e, old) in entities {
            if entity::exists(old) {
                entity::despawn_recursive(old);
            }
        }
    });

    spawn_query((position(), altitude()))
        .requires(is_medium_crop())
        .bind(move |entities| {
            for (e, (position, altitude)) in entities {
                // pseudo-randomly generate the angle so that when a crop on
                // this tile grows it doesn't also rotate
                let angle = position.dot(vec2(12.9898, 78.233)) * 43758.5453;

                entity::add_components(
                    e,
                    make_transformable()
                        .with(translation(), position.extend(altitude))
                        .with(rotation(), Quat::from_rotation_z(angle))
                        .with(local_to_world(), Mat4::IDENTITY),
                );
            }
        });

    spawn_query(model_prefab_url())
        .requires(is_medium_crop())
        .bind(move |entities| {
            for (e, prefab_url) in entities {
                let model = Entity::new()
                    .with(prefab_from_url(), prefab_url)
                    .with(local_to_parent(), Mat4::IDENTITY)
                    .spawn();

                // inherit old crop reference to model
                if let Some(despawn) = entity::get_component(e, despawn_when_loaded()) {
                    entity::add_component(model, despawn_when_loaded(), despawn);
                }

                entity::add_child(e, model);
            }
        });

    despawn_query(medium_crop_occupant()).bind(move |entities| {
        for (_, occupant) in entities {
            if !occupant.is_null() {
                entity::despawn_recursive(occupant);
            }
        }
    });
}
