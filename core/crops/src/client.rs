use ambient_api::{
    core::{
        prefab::components::spawned,
        transform::{
            components::{local_to_world, rotation, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use flowerpot_common::{impl_remote_update, RemoteEntityStore};
use packages::{
    map::components::position,
    region_networking::components::remote_entity,
    terrain::components::altitude,
    this::{components::*, messages::*},
};

mod shared;

impl_remote_update!(UpdateCropCoords);

#[main]
fn main() {
    shared::init_shared();

    let remote_store = RemoteEntityStore::new(remote_entity());

    remote_store.subscribe_update::<UpdateCropCoords>(move |e, data| {
        entity::add_component(e, coords(), data.position);
    });

    spawn_query(coords())
        .requires(is_medium_crop())
        .bind(move |entities| {
            for (e, coords) in entities {
                let new_position = coords.as_vec2() + 0.5;
                entity::add_component(e, position(), new_position);
            }
        });

    // despawn old crops once the new one has finished loading
    spawn_query((despawn_when_loaded(), spawned())).bind(move |entities| {
        for (e, (old, _)) in entities {
            if entity::exists(old) {
                entity::remove_component(e, despawn_when_loaded());
                entity::despawn_recursive(old);
            }
        }
    });

    // despawn old crops if the new crop hasn't finished loading
    despawn_query(despawn_when_loaded()).bind(move |entities| {
        for (_e, old) in entities {
            if entity::exists(old) {
                entity::despawn_recursive(old);
            }
        }
    });

    spawn_query((position(), altitude()))
        .requires(is_medium_crop())
        .bind(move |entities| {
            for (e, (position, altitude)) in entities {
                // pseudo-randomly generate the angle so that when a crop on
                // this tile grows it doesn't also rotate
                let angle = position.dot(vec2(12.9898, 78.233)) * 43758.5453;

                entity::add_components(
                    e,
                    make_transformable()
                        .with(translation(), position.extend(altitude))
                        .with(rotation(), Quat::from_rotation_z(angle))
                        .with(local_to_world(), Mat4::IDENTITY),
                );
            }
        });

    despawn_query(medium_crop_occupant()).bind(move |entities| {
        for (_, occupant) in entities {
            if !occupant.is_null() {
                entity::despawn_recursive(occupant);
            }
        }
    });
}
