use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::quad,
        transform::{lookat_target, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

use components::map::*;
use messages::{LoadChunk, Ready};

pub const CHUNK_SIZE: usize = 16;

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::ONE * 5.)
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .spawn();

    println!("Hello, Ambient!");

    for x in -2..=2 {
        for y in -2..=2 {
            let position = IVec2::new(x, y);

            let tile_num = CHUNK_SIZE * CHUNK_SIZE;
            let mut tiles = Vec::with_capacity(tile_num);
            for _ in 0..tile_num {
                tiles.push(Entity::new().spawn());
            }

            Entity::new()
                .with(chunk(), position)
                .with(chunk_tile_refs(), tiles)
                .spawn();
        }
    }

    let all_chunks = query(chunk()).build();
    Ready::subscribe(move |source, _| {
        let uid = source.client_user_id().unwrap();
        let chunks = all_chunks.evaluate();
        println!("Updating client {} with {} chunks", uid, chunks.len());
        for (_e, position) in chunks.iter() {
            LoadChunk::new(*position).send_client_targeted_unreliable(uid.clone());
        }
    });
}
