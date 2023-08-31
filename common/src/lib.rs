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
    fn on_message(&self, cb: impl FnMut(&mut T, Source, Message) + 'static);

    fn on_local_message(&self, cb: impl FnMut(&mut T, EntityId, Message) + 'static);

    #[cfg(feature = "server")]
    fn on_client_message(&self, cb: impl FnMut(&mut T, EntityId, Message) + 'static);
}

impl<T, Message> ActorExt<T, Message> for Arc<Mutex<T>>
where
    T: Send + Sync + 'static,
    Message: ModuleMessage,
{
    fn on_message(&self, mut cb: impl FnMut(&mut T, Source, Message) + 'static) {
        let actor = self.to_owned();
        Message::subscribe(move |context, data| {
            let mut actor = actor.lock().unwrap();
            cb(&mut actor, context.source, data);
        });
    }

    fn on_local_message(&self, mut cb: impl FnMut(&mut T, EntityId, Message) + 'static) {
        self.on_message(move |store, source, data| {
            if let Some(local) = source.local() {
                cb(store, local, data);
            }
        });
    }

    #[cfg(feature = "server")]
    fn on_client_message(&self, mut cb: impl FnMut(&mut T, EntityId, Message) + 'static) {
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
        cb: impl FnMut(&mut T, EntityId, Components::Data) + 'static,
    );

    fn on_event(
        &self,
        query: EventQuery<Components>,
        cb: impl FnMut(&mut T, EntityId, Components::Data) + 'static,
    );

    fn on_change(
        &self,
        query: ChangeQuery<Components>,
        cb: impl FnMut(&mut T, EntityId, Components::Data) + 'static,
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
        mut cb: impl FnMut(&mut T, EntityId, Components::Data) + 'static,
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
        mut cb: impl FnMut(&mut T, EntityId, Components::Data) + 'static,
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
        mut cb: impl FnMut(&mut T, EntityId, Components::Data) + 'static,
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

pub trait RemoteUpdate: ModuleMessage {
    fn get_remote_entity(&self) -> EntityId;
}

#[derive(Clone)]
pub struct RemoteEntityStore {
    inner: Arc<Mutex<HashMap<EntityId, EntityId>>>,
}

impl RemoteEntityStore {
    pub fn new(remote_entity: Component<EntityId>) -> Self {
        let inner = Default::default();
        let store = Self { inner };

        spawn_query(remote_entity).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.inner.lock().unwrap();
                for (e, remote) in entities {
                    store.insert(remote, e);
                }
            }
        });

        despawn_query(remote_entity).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.inner.lock().unwrap();
                for (_, remote) in entities {
                    store.remove(&remote);
                }
            }
        });

        store
    }

    pub fn remote_to_local(&self, remote: EntityId) -> Option<EntityId> {
        self.inner.lock().unwrap().get(&remote).copied()
    }

    pub fn subscribe_update<T: RemoteUpdate>(&self, mut cb: impl FnMut(EntityId, T) + 'static) {
        let store = self.to_owned();
        T::subscribe(move |_, data| {
            let remote = data.get_remote_entity();
            let Some(local) = store.remote_to_local(remote) else {
                return;
            };

            // sanity check in case of race condition with DespawnFauna
            if entity::exists(local) {
                cb(local, data);
            }
        });
    }
}

#[macro_export]
macro_rules! impl_remote_update {
    ($message:ident) => {
        impl ::flowerpot_common::RemoteUpdate for $message {
            fn get_remote_entity(&self) -> EntityId {
                self.thing
            }
        }
    };
}
