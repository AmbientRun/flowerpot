use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    components::core::{primitives::quad, rendering::color, transform::translation},
    concepts::make_transformable,
    prelude::*,
};

use components::map::*;
use messages::{LoadChunk, Ready, UnloadChunk};

// TODO deduplicate this
pub const CHUNK_SIZE: usize = 16;

#[main]
pub fn main() {
    let chunks = Arc::new(Mutex::new(HashMap::<IVec2, EntityId>::new()));

    LoadChunk::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            println!("Loading chunk: {}", data.pos);

            let mut tiles = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
            let tile_offset = data.pos.extend(0).as_vec3() * CHUNK_SIZE as f32;
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let tile_color = (x + y) & 1 == 1;
                    tiles.push(
                        Entity::new()
                            .with_merge(make_transformable())
                            .with(
                                translation(),
                                tile_offset + Vec3::new(x as f32, y as f32, 0.0),
                            )
                            .with(
                                color(),
                                if tile_color {
                                    vec4(1.0, 0.0, 0.0, 1.0)
                                } else {
                                    vec4(0.0, 1.0, 0.0, 1.0)
                                },
                            )
                            .with_default(quad())
                            .spawn(),
                    );
                }
            }

            let chunk = Entity::new()
                .with(chunk(), data.pos)
                .with(chunk_tile_refs(), tiles)
                .spawn();

            // TODO handle existing chunks
            chunks.lock().unwrap().insert(data.pos, chunk);
        }
    });

    UnloadChunk::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            println!("Unloading chunk: {}", data.pos);

            let chunk = chunks.lock().unwrap().remove(&data.pos).unwrap();

            for tile in entity::get_component(chunk, chunk_tile_refs()).unwrap_or_default() {
                entity::despawn_recursive(tile);
            }

            entity::despawn_recursive(chunk);
        }
    });

    println!("ready!");
    Ready::new().send_server_reliable();
}
