use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    ecs::{ChangeQuery, ComponentsTuple, EventQuery, GeneralQuery},
    message::Source,
    prelude::*,
};

pub const CHUNK_SIZE: usize = 16;

/// A helper type to map positions to entities with those positions.
///
/// Init with [init_map].
pub type PositionMap = Arc<Mutex<HashMap<IVec2, EntityId>>>;

/// Initializes a [PositionMap] using the given position component.
///
/// Returns a new [PositionMap] and creates queries that update it.
pub fn init_map(position_component: Component<IVec2>) -> PositionMap {
    let chunks = PositionMap::default();

    chunks.on_event(
        spawn_query(position_component),
        move |chunks, e, chunk_xy| {
            chunks.insert(chunk_xy, e);
        },
    );

    chunks.on_event(
        despawn_query(position_component),
        move |chunks, _e, chunk_xy| {
            chunks.remove(&chunk_xy);
        },
    );

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

/// Extension trait to easily build actors.
pub trait ActorExt<T, Message> {
    fn on_message(&self, cb: impl Fn(&mut T, Source, Message) + 'static);

    fn on_local_message(&self, cb: impl Fn(&mut T, EntityId, Message) + 'static);

    #[cfg(feature = "server")]
    fn on_client_message(&self, cb: impl Fn(&mut T, EntityId, Message) + 'static);
}

impl<T, Message> ActorExt<T, Message> for Arc<Mutex<T>>
where
    T: Send + Sync + 'static,
    Message: ModuleMessage,
{
    fn on_message(&self, cb: impl Fn(&mut T, Source, Message) + 'static) {
        let actor = self.to_owned();
        Message::subscribe(move |source, data| {
            let mut actor = actor.lock().unwrap();
            cb(&mut actor, source, data);
        });
    }

    fn on_local_message(&self, cb: impl Fn(&mut T, EntityId, Message) + 'static) {
        self.on_message(move |store, source, data| {
            if let Some(local) = source.local() {
                cb(store, local, data);
            }
        });
    }

    #[cfg(feature = "server")]
    fn on_client_message(&self, cb: impl Fn(&mut T, EntityId, Message) + 'static) {
        self.on_message(move |store, source, data| {
            if let Some(local) = source.client_entity_id() {
                cb(store, local, data);
            }
        });
    }
}

/// Extension trait to easily build impure ECS systems.
pub trait SystemExt<T, Components: ComponentsTuple + Copy + Clone + 'static> {
    fn each_frame(
        &self,
        query: GeneralQuery<Components>,
        cb: impl Fn(&mut T, EntityId, Components::Data) + 'static,
    );

    fn on_event(
        &self,
        query: EventQuery<Components>,
        cb: impl Fn(&mut T, EntityId, Components::Data) + 'static,
    );

    fn on_change(
        &self,
        query: ChangeQuery<Components>,
        cb: impl Fn(&mut T, EntityId, Components::Data) + 'static,
    );
}

impl<T, Components> SystemExt<T, Components> for Arc<Mutex<T>>
where
    T: Send + Sync + 'static,
    Components: ComponentsTuple + Copy + Clone + 'static,
{
    fn each_frame(
        &self,
        query: GeneralQuery<Components>,
        cb: impl Fn(&mut T, EntityId, Components::Data) + 'static,
    ) {
        let system = self.to_owned();
        query.each_frame(move |entities| {
            let mut system = system.lock().unwrap();
            for (e, components) in entities {
                cb(&mut system, e, components);
            }
        });
    }

    fn on_event(
        &self,
        query: EventQuery<Components>,
        cb: impl Fn(&mut T, EntityId, Components::Data) + 'static,
    ) {
        let system = self.to_owned();
        query.bind(move |entities| {
            let mut system = system.lock().unwrap();
            for (e, components) in entities {
                cb(&mut system, e, components);
            }
        });
    }

    fn on_change(
        &self,
        query: ChangeQuery<Components>,
        cb: impl Fn(&mut T, EntityId, Components::Data) + 'static,
    ) {
        let system = self.to_owned();
        query.bind(move |entities| {
            let mut system = system.lock().unwrap();
            for (e, components) in entities {
                cb(&mut system, e, components);
            }
        });
    }
}
