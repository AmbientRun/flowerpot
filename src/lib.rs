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

/// A utility function to diff two sorted iterators.
pub fn diff_sorted<'a, V>(
    mut a_iter: impl Iterator<Item = V>,
    mut b_iter: impl Iterator<Item = V>,
    mut on_a: impl FnMut(&V),
    mut on_b: impl FnMut(&V),
) where
    V: Ord,
{
    let mut current_a = a_iter.next();
    let mut current_b = b_iter.next();

    loop {
        use std::cmp::Ordering;
        match (current_a.as_ref(), current_b.as_ref()) {
            (Some(a), Some(b)) => match a.cmp(&b) {
                Ordering::Less => {
                    on_a(a);
                    current_a = a_iter.next();
                }
                Ordering::Equal => {
                    current_a = a_iter.next();
                    current_b = b_iter.next();
                }
                Ordering::Greater => {
                    on_b(b);
                    current_b = b_iter.next();
                }
            },
            (Some(a), None) => {
                on_a(a);
                current_a = a_iter.next();
            }
            (None, Some(b)) => {
                on_b(b);
                current_b = b_iter.next();
            }
            (None, None) => break,
        }
    }
}
