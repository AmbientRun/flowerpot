use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    core::{
        prefab::components::prefab_from_url,
        transform::{components::*, concepts::make_transformable},
    },
    prelude::*,
};

mod shared;

use packages::{
    map::components::position,
    nameplate::concepts::make_nameplate,
    region_networking::components::remote_entity,
    terrain::components::altitude,
    this::{components::*, messages::*},
};

#[main]
fn main() {
    let store = ThingStore::new();

    store.subscribe_update::<UpdateFaunaPosition>(move |e, data| {
        entity::add_component(e, position(), data.position);
    });

    store.subscribe_update::<UpdateFaunaPitch>(move |e, data| {
        entity::add_component(e, pitch(), data.pitch);
    });

    store.subscribe_update::<UpdateFaunaYaw>(move |e, data| {
        entity::add_component(e, yaw(), data.yaw);
    });

    store.subscribe_update::<UpdateFaunaName>(move |e, data| {
        entity::add_components(
            e,
            Entity::new()
                .with_merge(make_nameplate())
                .with(name(), data.name.clone())
                .with(packages::nameplate::components::name(), data.name),
        );
    });

    spawn_query((remote_entity(), position(), altitude(), model_prefab_url()))
        .requires(is_fauna())
        .bind(move |entities| {
            for (e, (remote, position, height, prefab_url)) in entities {
                if remote == player::get_local() {
                    // don't spawn a prefab for the local player
                    continue;
                }

                entity::add_components(
                    e,
                    make_transformable()
                        .with(translation(), position.extend(height))
                        .with(prefab_from_url(), prefab_url),
                );
            }
        });

    change_query((position(), altitude()))
        .track_change(position())
        .requires(is_fauna())
        .bind(move |entities| {
            for (e, (position, height)) in entities {
                entity::add_component(e, translation(), position.extend(height));
            }
        });

    change_query(yaw())
        .track_change(position())
        .requires((is_fauna(), rotation()))
        .bind(move |entities| {
            for (e, yaw) in entities {
                let new_rotation = Quat::from_rotation_z(yaw);
                entity::set_component(e, rotation(), new_rotation);
            }
        });

    eprintln!("fauna mod loaded");
    entity::add_component(entity::resources(), is_mod_loaded(), ());
}

pub trait ThingUpdate: ModuleMessage {
    fn get_remote_entity(&self) -> EntityId;
}

macro_rules! impl_update_for_thing {
    ($message:ident) => {
        impl ThingUpdate for $message {
            fn get_remote_entity(&self) -> EntityId {
                self.thing
            }
        }
    };
}

impl_update_for_thing!(UpdateFaunaPosition);
impl_update_for_thing!(UpdateFaunaPitch);
impl_update_for_thing!(UpdateFaunaYaw);
impl_update_for_thing!(UpdateFaunaName);

#[derive(Clone)]
pub struct ThingStore {
    inner: Arc<Mutex<HashMap<EntityId, EntityId>>>,
}

impl ThingStore {
    pub fn new() -> Self {
        let inner = Default::default();
        let store = Self { inner };

        spawn_query((is_fauna(), remote_entity())).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.inner.lock().unwrap();
                for (e, (_, remote)) in entities {
                    store.insert(remote, e);
                }
            }
        });

        despawn_query((is_fauna(), remote_entity())).bind({
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

    pub fn subscribe_update<T: ThingUpdate>(&self, mut cb: impl FnMut(EntityId, T) + 'static) {
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
