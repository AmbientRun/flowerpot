use ambient_api::prelude::*;
use flowerpot_common::{ActorExt, CHUNK_SIZE};

use packages::map::{components::*, messages::*};

mod shared;

#[main]
pub fn main() {
    let chunks = shared::init_shared_map();

    chunks.on_message(move |_chunks, _, data: LoadChunk| {
        println!("Loading chunk: {}", data.pos);

        let mut tiles = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        for _y in 0..CHUNK_SIZE {
            for _x in 0..CHUNK_SIZE {
                tiles.push(Entity::new().spawn());
            }
        }

        Entity::new()
            .with(chunk(), data.pos)
            .with(chunk_tile_refs(), tiles)
            .spawn();
    });

    chunks.on_message(move |chunks, _, data: UnloadChunk| {
        println!("Unloading chunk: {}", data.pos);

        let Some(chunk) = chunks.remove(&data.pos) else {
            return;
        };

        for tile in entity::get_component(chunk, chunk_tile_refs()).unwrap_or_default() {
            entity::despawn_recursive(tile);
        }

        entity::despawn_recursive(chunk);
    });

    entity::add_component(entity::resources(), is_mod_loaded(), ());
}
