use ambient_api::prelude::*;
use flowerpot_common::{init_map, PositionMap, SystemExt, CHUNK_SIZE};

use crate::packages::map::components::*;

pub fn init_shared_map() -> PositionMap {
    let chunks = init_map(chunk());

    chunks.on_change(
        change_query(position()).track_change(position()),
        move |chunks, e, position| {
            let xy = (position / CHUNK_SIZE as f32).floor().as_ivec2();
            if let Some(chunk) = chunks.get(&xy) {
                entity::add_component(e, in_chunk(), *chunk);
            }
        },
    );

    chunks.on_event(
        despawn_query((in_chunk(), position())),
        move |_chunks, e, _| {
            if entity::exists(e) {
                entity::remove_component(e, in_chunk());
            }
        },
    );

    chunks
}
