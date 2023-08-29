use ambient_api::prelude::*;

use crate::packages::{
    map::components::{chunk, chunk_tile_index, chunk_tile_refs, in_chunk},
    this::components::*,
};
use flowerpot_common::{init_map, SystemExt, CHUNK_SIZE};

pub fn init_shared() {
    let chunks = init_map(chunk());

    chunks.on_event(
        spawn_query(coords()).requires(is_medium_crop()),
        move |chunks, e, coords| {
            let xy = coords / CHUNK_SIZE as i32;
            if let Some(chunk) = chunks.get(&xy) {
                println!("{} at {}", e, coords);

                let fine = coords - xy * CHUNK_SIZE as i32;
                let tile_idx = fine.y * CHUNK_SIZE as i32 + fine.x;

                let Ok(tile_idx) = TryInto::<u8>::try_into(tile_idx) else {
                    eprintln!("tile index {} is out-of-bounds", tile_idx);
                    return;
                };

                let tiles = entity::get_component(*chunk, chunk_tile_refs()).unwrap();
                let tile = tiles[tile_idx as usize];

                let old_occupant =
                    entity::get_component(tile, medium_crop_occupant()).unwrap_or_default();

                if !old_occupant.is_null() {
                    entity::despawn_recursive(old_occupant);
                }

                entity::add_components(
                    e,
                    Entity::new()
                        .with(in_chunk(), *chunk)
                        .with(chunk_tile_index(), tile_idx)
                        .with(on_tile(), tile)
                        .with(despawn_when_loaded(), old_occupant)
                        .with(age(), 0),
                );

                entity::add_component(tile, medium_crop_occupant(), e);
            }
        },
    );

    despawn_query(on_tile())
        .requires(is_medium_crop())
        .bind(move |entities| {
            for (e, tile) in entities {
                if entity::get_component(tile, medium_crop_occupant()) == Some(e) {
                    entity::set_component(tile, medium_crop_occupant(), EntityId::null());
                }
            }
        });
}
