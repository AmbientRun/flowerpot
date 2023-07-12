use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use ambient_api::prelude::*;

use crate::components::map::*;

pub const CHUNK_SIZE: usize = 16;

pub type ChunkMap = Arc<RwLock<HashMap<IVec2, EntityId>>>;

pub fn init_shared_map() -> ChunkMap {
    let chunks = ChunkMap::default();

    spawn_query(chunk()).bind({
        let chunks = chunks.clone();
        move |entities| {
            let mut chunks = chunks.write().unwrap();
            for (e, chunk_xy) in entities {
                chunks.insert(chunk_xy, e);
            }
        }
    });

    despawn_query(chunk()).bind({
        let chunks = chunks.clone();
        move |entities| {
            let mut chunks = chunks.write().unwrap();
            for (_, chunk_xy) in entities {
                chunks.remove(&chunk_xy);
            }
        }
    });

    change_query(position()).track_change(position()).bind({
        let chunks = chunks.clone();
        move |entities| {
            let chunks = chunks.read().unwrap();
            for (e, position) in entities {
                let xy = (position / CHUNK_SIZE as f32).as_ivec2();
                let Some(chunk) = chunks.get(&xy) else { continue };
                entity::add_component(e, in_chunk(), *chunk);
            }
        }
    });

    despawn_query(position()).bind(move |entities| {
        for (e, _) in entities {
            entity::remove_component(e, in_chunk());
        }
    });

    chunks
}
