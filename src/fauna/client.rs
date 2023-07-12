use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    components::core::{primitives::sphere_radius, rendering::color, transform::translation},
    concepts::{make_sphere, make_transformable},
    prelude::*,
};

mod shared;

use components::{fauna::*, map::position, terrain};
use messages::*;

#[main]
fn main() {
    SpawnFauna::subscribe(move |_, data| {
        Entity::new()
            .with_default(fauna())
            .with(remote_entity(), data.eid)
            .spawn();
    });

    let store = FaunaStore::new();

    store.subscribe_update::<DespawnFauna>(move |e, _| {
        entity::despawn_recursive(e);
    });

    store.subscribe_update::<UpdateFaunaPosition>(move |e, data| {
        entity::add_component(e, position(), data.position);
    });

    store.subscribe_update::<UpdateFaunaYaw>(move |e, data| {
        entity::add_component(e, yaw(), data.yaw);
    });

    // temp fauna rendering code

    spawn_query((fauna(), position())).bind(move |entities| {
        for (e, (_, position)) in entities {
            entity::add_components(
                e,
                make_transformable()
                    .with(translation(), position.extend(0.0))
                    .with_merge(make_sphere())
                    .with(sphere_radius(), 0.2)
                    .with(color(), vec4(1.0, 1.0, 0.0, 1.0)),
            );
        }
    });

    change_query((fauna(), position()))
        .track_change(position())
        .bind(move |entities| {
            for (e, (_, position)) in entities {
                entity::add_component(e, translation(), position.extend(0.0));
            }
        });

    eprintln!("fauna mod loaded");
    entity::add_component(entity::resources(), mod_loaded(), ());
}

pub trait FaunaUpdate: ModuleMessage {
    fn get_remote_entity(&self) -> EntityId;
}

macro_rules! impl_update_for_eid {
    ($message:ident) => {
        impl FaunaUpdate for $message {
            fn get_remote_entity(&self) -> EntityId {
                self.eid
            }
        }
    };
}

impl_update_for_eid!(DespawnFauna);
impl_update_for_eid!(UpdateFaunaPosition);
impl_update_for_eid!(UpdateFaunaYaw);

#[derive(Clone)]
pub struct FaunaStore {
    inner: Arc<Mutex<HashMap<EntityId, EntityId>>>,
}

impl FaunaStore {
    pub fn new() -> Self {
        let inner = Default::default();
        let store = Self { inner };

        spawn_query((fauna(), remote_entity())).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.inner.lock().unwrap();
                for (e, (_, remote)) in entities {
                    store.insert(remote, e);
                }
            }
        });

        despawn_query((fauna(), remote_entity())).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.inner.lock().unwrap();
                for (_, (_, remote)) in entities {
                    store.remove(&remote);
                }
            }
        });

        store
    }

    pub fn remote_to_local(&self, remote: EntityId) -> Option<EntityId> {
        self.inner.lock().unwrap().get(&remote).copied()
    }

    pub fn subscribe_update<T: FaunaUpdate>(&self, mut cb: impl FnMut(EntityId, T) + 'static) {
        let store = self.to_owned();
        T::subscribe(move |_, data| {
            let remote = data.get_remote_entity();
            let Some(local) = store.remote_to_local(remote) else { return };
            cb(local, data);
        });
    }
}
