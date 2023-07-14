use ambient_api::prelude::*;

use components::map::*;
use flowerpot::CHUNK_SIZE;
use messages::{LoadChunk, Ready, UnloadChunk};

mod shared;

#[main]
pub fn main() {
    let chunks = shared::init_shared_map();

    LoadChunk::subscribe(move |_, data| {
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

    UnloadChunk::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            println!("Unloading chunk: {}", data.pos);

            let chunk = chunks.write().unwrap().remove(&data.pos).unwrap();

            for tile in entity::get_component(chunk, chunk_tile_refs()).unwrap_or_default() {
                entity::despawn_recursive(tile);
            }

            entity::despawn_recursive(chunk);
        }
    });

    println!("ready!");
    Ready::new().send_server_reliable();
}
