use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    core::{
        app::components::main_scene,
        ecs::components::children,
        primitives::{components::sphere_radius, concepts::make_sphere},
        rendering::components::color,
        text::types::FontStyle,
        transform::{components::*, concepts::make_transformable},
    },
    prelude::*,
};

mod shared;

use embers::{
    fauna::{components::*, messages::*},
    map::components::position,
    terrain::components::altitude,
};

#[main]
fn main() {
    let store = FaunaStore::new(true);

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

        use ambient_api::core::text::components::*;
        let name = data.name;
        const APPROXIMATE_CHAR_WIDTH: f32 = 36.0;
        let width = name.chars().count() as f32 * APPROXIMATE_CHAR_WIDTH;
        let transform = Mat4::from_scale(Vec3::ONE * 0.005)
            * Mat4::from_rotation_x(180_f32.to_radians())
            * Mat4::from_translation(Vec3::new(-width / 2.0, 0.0, 0.0));

        if let Some(container) = entity::get_component(e, name_container()) {
            for child in entity::get_component(container, children()).unwrap_or_default() {
                entity::add_component(child, text(), name.clone());
                entity::add_component(child, local_to_parent(), transform);
            }
        } else {
            let display_name = Entity::new()
                .with(local_to_parent(), transform)
                .with(text(), name)
                .with(font_size(), 72.0)
                .with(
                    font_family(),
                    "https://github.com/madmalik/mononoki/raw/main/export/mononoki-Regular.ttf"
                        .to_string(),
                )
                .with(font_style(), FontStyle::Regular)
                .with(color(), vec4(1.0, 1.0, 1.0, 1.0))
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

    spawn_query((is_fauna(), position(), altitude())).bind(move |entities| {
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

    change_query((is_fauna(), position(), altitude()))
        .track_change(position())
        .bind(move |entities| {
            for (e, (_, position, height)) in entities {
                entity::add_component(e, translation(), position.extend(height));
            }
        });

    query((is_fauna(), position(), altitude(), name_container())).each_frame(move |entities| {
        for (_e, (_, position, height, container)) in entities {
            entity::add_component(container, translation(), position.extend(height + 2.5));
        }
    });

    despawn_query((is_fauna(), name_container())).bind(move |entities| {
        for (_e, (_, container)) in entities {
            entity::despawn_recursive(container);
        }
    });

    eprintln!("fauna mod loaded");
    entity::add_component(entity::resources(), is_mod_loaded(), ());
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
    pub fn new(owns_remote_entitites: bool) -> Self {
        let inner = Default::default();
        let store = Self { inner };

        if owns_remote_entitites {
            SpawnFauna::subscribe({
                let store = store.clone();
                move |_, data| {
                    let mut store = store.inner.lock().unwrap();

                    if store.contains_key(&data.eid) {
                        return;
                    }

                    let e = Entity::new()
                        .with_default(is_fauna())
                        .with(remote_entity(), data.eid)
                        .spawn();

                    store.insert(data.eid, e);
                }
            });
        } else {
            spawn_query((is_fauna(), remote_entity())).bind({
                let store = store.clone();
                move |entities| {
                    let mut store = store.inner.lock().unwrap();
                    for (e, (_, remote)) in entities {
                        store.insert(remote, e);
                    }
                }
            });
        }

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

    pub fn subscribe_update<T: FaunaUpdate>(&self, mut cb: impl FnMut(EntityId, T) + 'static) {
        let store = self.to_owned();
        T::subscribe(move |_, data| {
            let remote = data.get_remote_entity();
            let Some(local) = store.remote_to_local(remote) else { return };

            // sanity check in case of race condition with DespawnFauna
            if entity::exists(local) {
                cb(local, data);
            }
        });
    }
}
