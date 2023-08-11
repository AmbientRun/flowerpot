use ambient_api::prelude::*;
use flowerpot_common::{init_map, PositionMap, CHUNK_SIZE};

use crate::embers::map::components::*;

pub fn init_shared_map() -> PositionMap {
    let chunks = init_map(chunk());

    change_query(position()).track_change(position()).bind({
        let chunks = chunks.clone();
        move |entities| {
            let chunks = chunks.read().unwrap();
            for (e, position) in entities {
                let xy = (position / CHUNK_SIZE as f32).floor().as_ivec2();
                let Some(chunk) = chunks.get(&xy) else { continue };
                entity::add_component(e, in_chunk(), *chunk);
            }
        }
    });

    despawn_query((in_chunk(), position())).bind(move |entities| {
        for (e, _) in entities {
            if entity::exists(e) {
                entity::remove_component(e, in_chunk());
            }
        }
    });

    chunks
}
