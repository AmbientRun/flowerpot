use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use ambient_api::prelude::*;

pub const CHUNK_SIZE: usize = 16;

/// A helper type to map positions to entities with those positions.
///
/// Init with [init_map].
pub type PositionMap = Arc<RwLock<HashMap<IVec2, EntityId>>>;

/// Initializes a [PositionMap] using the given position component.
///
/// Returns a new [PositionMap] and creates queries that update it.
pub fn init_map(position_component: Component<IVec2>) -> PositionMap {
    let chunks = PositionMap::default();

    spawn_query(position_component).bind({
        let chunks = chunks.clone();
        move |entities| {
            let mut chunks = chunks.write().unwrap();
            for (e, chunk_xy) in entities {
                chunks.insert(chunk_xy, e);
            }
        }
    });

    despawn_query(position_component).bind({
        let chunks = chunks.clone();
        move |entities| {
            let mut chunks = chunks.write().unwrap();
            for (_, chunk_xy) in entities {
                chunks.remove(&chunk_xy);
            }
        }
    });

    chunks
}
