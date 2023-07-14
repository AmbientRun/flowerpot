use ambient_api::prelude::*;

use components::map::*;
use messages::{LoadChunk, OnPlayerLoadChunk, Ready};

mod shared;

use shared::CHUNK_SIZE;

#[main]
pub fn main() {
    shared::init_shared_map();

    for x in -8..=8 {
        for y in -8..=8 {
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
        let player = source.client_entity_id().unwrap();
        let uid = source.client_user_id().unwrap();
        let chunks = all_chunks.evaluate();
        println!("Updating client {} with {} chunks", uid, chunks.len());
        for (e, position) in chunks {
            LoadChunk::new(position).send_client_targeted_reliable(uid.clone());
            OnPlayerLoadChunk::new(e, position, player, uid.clone()).send_local_broadcast(true);
        }
    });
}
