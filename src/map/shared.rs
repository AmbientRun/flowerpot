use ambient_api::prelude::*;
use flowerpot::{init_map, PositionMap, CHUNK_SIZE};

use crate::components::map::*;

pub fn init_shared_map() -> PositionMap {
    let chunks = init_map(chunk());

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
