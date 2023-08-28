use ambient_api::{
    core::transform::{components::*, concepts::make_transformable},
    prelude::*,
};

mod shared;

use flowerpot_common::{impl_remote_update, RemoteEntityStore};
use packages::{
    map::components::position,
    nameplate::concepts::make_nameplate,
    region_networking::components::remote_entity,
    terrain::components::altitude,
    this::{components::*, messages::*},
};

#[main]
fn main() {
    let store = RemoteEntityStore::new(remote_entity());

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

    spawn_query((position(), altitude()))
        .requires(is_fauna())
        .bind(move |entities| {
            for (e, (position, altitude)) in entities {
                entity::add_components(
                    e,
                    make_transformable().with(translation(), position.extend(altitude)),
                );
            }
        });

    change_query((position(), altitude()))
        .track_change(position())
        .requires(is_fauna())
        .bind(move |entities| {
            for (e, (position, altitude)) in entities {
                entity::add_component(e, translation(), position.extend(altitude));
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

impl_remote_update!(UpdateFaunaPosition);
impl_remote_update!(UpdateFaunaPitch);
impl_remote_update!(UpdateFaunaYaw);
impl_remote_update!(UpdateFaunaName);
