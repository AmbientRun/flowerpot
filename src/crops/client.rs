use ambient_api::{
    components::core::{
        prefab::prefab_from_url,
        transform::{local_to_parent, local_to_world, rotation, translation},
    },
    concepts::make_transformable,
    prelude::*,
};

use components::{
    crops::*,
    map::{chunk_tile_index, in_chunk, position},
    terrain::height,
};
use flowerpot::CHUNK_SIZE;
use messages::UpdateMediumCrops;

mod shared;

#[main]
fn main() {
    spawn_query(components::map::chunk()).bind(move |entities| {
        for (e, _) in entities {
            let tile_num = CHUNK_SIZE * CHUNK_SIZE;
            let occupants = vec![EntityId::null(); tile_num];
            entity::add_component(e, medium_crop_occupants(), occupants.clone());
        }
    });

    let chunks = flowerpot::init_map(components::map::chunk());

    UpdateMediumCrops::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            let chunks = chunks.clone();
            run_async(async move {
                let mut tries = std::iter::repeat(()).take(20);
                let chunk = loop {
                    if let Some(_) = tries.next() {
                        if let Some(chunk) = chunks.read().unwrap().get(&data.chunk).copied() {
                            break Some(chunk);
                        }
                    } else {
                        eprintln!(
                            "warning: dropping medium crop update on unloaded chunk {}",
                            data.chunk
                        );

                        break None;
                    }

                    sleep(0.1).await;
                };

                let Some(chunk) = chunk else { return };
                let Some(mut occupants) = entity::get_component(chunk, medium_crop_occupants()) else { return };

                eprintln!("update: {:#?}", data.crop_tiles);

                for (tile_idx, class) in data
                    .crop_tiles
                    .into_iter()
                    .zip(data.crop_classes.into_iter())
                {
                    eprintln!("updating {} with {:?}", tile_idx, class);

                    let occupant = &mut occupants[tile_idx as usize];

                    if !occupant.is_null() {
                        entity::despawn_recursive(*occupant);
                    }

                    if class.is_null() {
                        continue;
                    }

                    let occupant_position = (data.chunk * CHUNK_SIZE as i32).as_vec2()
                        + vec2(
                            (tile_idx as usize % CHUNK_SIZE) as f32,
                            (tile_idx as usize / CHUNK_SIZE) as f32,
                        )
                        + 0.5;

                    *occupant = entity::get_all_components(class)
                        .with_default(medium_crop())
                        .with(position(), occupant_position)
                        .with(in_chunk(), chunk)
                        .with(chunk_tile_index(), tile_idx)
                        .spawn();
                }

                entity::set_component(chunk, medium_crop_occupants(), occupants);
            });
        }
    });

    spawn_query((position(), height(), medium_crop(), model_prefab_path())).bind(move |entities| {
        for (e, (position, height, _, prefab_path)) in entities {
            let model = Entity::new()
                .with(prefab_from_url(), asset::url(prefab_path).unwrap())
                .with_default(local_to_parent())
                .spawn();

            // TODO deterministic angle using tile coordinates
            let angle = random::<f32>() * std::f32::consts::TAU;

            let transform = make_transformable()
                .with(translation(), position.extend(height))
                .with(rotation(), Quat::from_rotation_z(angle))
                .with_default(local_to_world())
                .spawn();

            entity::add_child(transform, model);
            entity::add_child(e, transform);
        }
    });

    despawn_query(medium_crop_occupants()).bind(move |entities| {
        for (_, occupants) in entities {
            for occupant in occupants {
                if !occupant.is_null() {
                    entity::despawn_recursive(occupant);
                }
            }
        }
    });
}
