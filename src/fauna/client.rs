use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    components::core::{
        app::main_scene,
        ecs::children,
        primitives::sphere_radius,
        rendering::color,
        transform::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, spherical_billboard,
            translation,
        },
    },
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

    store.subscribe_update::<UpdateFaunaName>(move |e, data| {
        entity::add_component(e, name(), data.name.clone());

        use ambient_api::components::core::text::*;
        let name = data.name;
        if let Some(container) = entity::get_component(e, name_container()) {
            for child in entity::get_component(container, children()).unwrap_or_default() {
                entity::add_component(child, text(), name.clone());
            }
        } else {
            let display_name = Entity::new()
                .with(
                    local_to_parent(),
                    Mat4::from_scale(Vec3::ONE * 0.02)
                        * Mat4::from_rotation_x(180_f32.to_radians()),
                )
                .with(text(), name)
                .with(font_size(), 36.0)
                .with(font_family(), "Default".to_string())
                .with(font_style(), "Regular".to_string())
                .with(color(), vec4(1.0, 0.0, 1.0, 1.0))
                .with_default(main_scene())
                .with_default(local_to_world())
                .with_default(mesh_to_local())
                .with_default(mesh_to_world())
                .spawn();

            let container = make_transformable()
                .with_default(main_scene())
                .with_default(local_to_world())
                .with_default(spherical_billboard())
                .spawn();

            entity::add_child(container, display_name);
            entity::add_component(e, name_container(), container);
        }
    });

    // temp fauna rendering code

    spawn_query((fauna(), position(), terrain::height())).bind(move |entities| {
        for (e, (_, position, height)) in entities {
            entity::add_components(
                e,
                make_transformable()
                    .with(translation(), position.extend(height))
                    .with_merge(make_sphere())
                    .with(sphere_radius(), 0.2)
                    .with(color(), vec4(1.0, 1.0, 0.0, 1.0)),
            );
        }
    });

    change_query((fauna(), position(), terrain::height()))
        .track_change(position())
        .bind(move |entities| {
            for (e, (_, position, height)) in entities {
                entity::add_component(e, translation(), position.extend(height));
            }
        });

    query((fauna(), position(), terrain::height(), name_container())).each_frame(move |entities| {
        for (_e, (_, position, height, container)) in entities {
            entity::add_component(container, translation(), position.extend(height + 2.5));
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
impl_update_for_eid!(UpdateFaunaName);

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
