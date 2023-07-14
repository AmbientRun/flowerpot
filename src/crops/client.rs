use ambient_api::{
    components::core::{
        primitives::cube,
        transform::{local_to_world, translation},
    },
    concepts::make_transformable,
    prelude::*,
};

use components::{
    crops::*,
    map::{chunk_tile_index, chunk_tile_refs, in_chunk, position},
    terrain::height,
};
use flowerpot::CHUNK_SIZE;
use messages::UpdateMediumCrops;

mod shared;

#[main]
fn main() {
    let chunks = flowerpot::init_map(components::map::chunk());

    UpdateMediumCrops::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            let chunks = chunks.read().unwrap();
            let chunk_pos = data.chunk;
            let Some(chunk) = chunks.get(&chunk_pos).copied() else { return };
            let Some(tiles) = entity::get_component(chunk, chunk_tile_refs()) else { return };
            for (tile_idx, class) in data
                .crop_tiles
                .into_iter()
                .zip(data.crop_classes.into_iter())
            {
                let tile = tiles[tile_idx as usize];

                if let Some(old_occupant) = entity::get_component(tile, medium_crop_occupant()) {
                    if !old_occupant.is_null() {
                        entity::despawn_recursive(old_occupant);
                    }
                }

                let occupant_position = (chunk_pos * CHUNK_SIZE as i32).as_vec2()
                    + vec2(
                        (tile_idx as usize % CHUNK_SIZE) as f32,
                        (tile_idx as usize / CHUNK_SIZE) as f32,
                    )
                    + 0.5;

                let new_occupant = entity::get_all_components(class)
                    .with_default(medium_crop())
                    .with(position(), occupant_position)
                    .with(in_chunk(), chunk)
                    .with(chunk_tile_index(), tile_idx)
                    .spawn();

                entity::add_component(tile, medium_crop_occupant(), new_occupant);
            }
        }
    });

    spawn_query((position(), height(), medium_crop())).bind(move |entities| {
        for (e, (position, height, _)) in entities {
            let model = make_transformable()
                .with_default(local_to_world())
                .with_default(cube())
                .with(translation(), position.extend(height))
                .spawn();

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
