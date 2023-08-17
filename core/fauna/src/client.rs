use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    core::{
        app::components::main_scene,
        ecs::components::children,
        prefab::components::prefab_from_url,
        primitives::{components::sphere_radius, concepts::make_sphere},
        rendering::components::color,
        text::types::FontStyle,
        transform::{components::*, concepts::make_transformable},
    },
    prelude::*,
};

mod shared;

use packages::{
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

    store.subscribe_update::<UpdateFaunaPitch>(move |e, data| {
        entity::add_component(e, pitch(), data.pitch);
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
                .with(main_scene(), ())
                .with(local_to_world(), Mat4::IDENTITY)
                .with(mesh_to_local(), Mat4::IDENTITY)
                .with(mesh_to_world(), Mat4::IDENTITY)
                .spawn();

            let container = make_transformable()
                .with(main_scene(), ())
                .with(local_to_world(), Mat4::IDENTITY)
                .with(spherical_billboard(), ())
                .spawn();

            entity::add_child(container, display_name);
            entity::add_component(e, name_container(), container);
        }
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
impl_update_for_eid!(UpdateFaunaPitch);
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

                    let base = if data.class.is_null() {
                        Entity::new()
                    } else {
                        entity::get_all_components(data.class)
                    };

                    let e = base
                        .with(is_fauna(), ())
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
