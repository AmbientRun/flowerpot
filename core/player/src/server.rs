use ambient_api::{
    core::player::components::{is_player, user_id},
    prelude::*,
};

use flowerpot_common::SystemExt;
use packages::{
    fauna::components::{is_fauna, pitch, yaw},
    map::components::{chunk, in_chunk, position},
    region_networking::messages::{LoadPlayerRegion, UnloadPlayerRegion},
    things::components::{class_ref, is_class, model_prefab_url},
    this::{assets::url, components::*, messages::*},
};

mod shared;

#[main]
fn main() {
    shared::init_shared_player();

    let player_class = Entity::new()
        .with(is_class(), ())
        .with(is_fauna(), ())
        .with(speed(), 1.0)
        .with(model_prefab_url(), url("player.glb"))
        .spawn();

    let chunks = flowerpot_common::init_map(chunk());

    spawn_query((is_player(), is_fauna())).bind(move |entities| {
        for (e, _) in entities {
            let left_hand = Entity::new().with(owner_ref(), e).spawn();
            let right_hand = Entity::new().with(owner_ref(), e).spawn();

            entity::add_components(
                e,
                Entity::new()
                    .with(class_ref(), player_class)
                    .with(position(), vec2(0.0, 0.0))
                    .with(direction(), vec2(0.0, 0.0))
                    .with(yaw(), 0.0)
                    .with(left_hand_ref(), left_hand)
                    .with(right_hand_ref(), right_hand)
                    .with(loaded_chunks(), vec![]),
            );
        }
    });

    chunks.on_event(
        despawn_query((user_id(), loaded_chunks())).requires(is_player()),
        move |chunks, player, (uid, loaded)| {
            for position in loaded {
                let Some(chunk) = chunks.get(&position) else {
                    continue;
                };

                UnloadPlayerRegion::new(*chunk, player, uid.clone()).send_local_broadcast(true);
            }
        },
    );

    chunks.on_change(
        change_query((user_id(), loaded_chunks(), in_chunk())).track_change(in_chunk()),
        move |chunks, e, (uid, old_chunks, current_chunk)| {
            let current_pos = entity::get_component(current_chunk, chunk()).unwrap();
            let mut new_chunks = Vec::new();
            for y in -4..4 {
                for x in -4..4 {
                    new_chunks.push(ivec2(x, y) + current_pos);
                }
            }

            // TODO this is hilariously slow. please use sorted diffs

            for new_chunk in new_chunks.iter() {
                if !old_chunks.contains(new_chunk) {
                    let Some(chunk) = chunks.get(new_chunk) else {
                        continue;
                    };

                    LoadPlayerRegion::new(*chunk, e, uid.clone()).send_local_broadcast(true);
                    LoadChunk::new(*new_chunk).send_client_targeted_reliable(uid.clone());
                }
            }

            for old_chunk in old_chunks.iter() {
                if !new_chunks.contains(old_chunk) {
                    let Some(chunk) = chunks.get(old_chunk) else {
                        continue;
                    };

                    UnloadPlayerRegion::new(*chunk, e, uid.clone()).send_local_broadcast(true);
                    UnloadChunk::new(*old_chunk).send_client_targeted_reliable(uid.clone());
                }
            }

            entity::set_component(e, loaded_chunks(), new_chunks);
        },
    );

    UpdatePlayerDirection::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else {
            return;
        };
        entity::add_component(e, direction(), data.direction.clamp_length_max(1.0));
    });

    UpdatePlayerAngle::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else {
            return;
        };
        entity::add_component(e, pitch(), data.pitch);
        entity::add_component(e, yaw(), data.yaw);
    });

    query(position())
        .requires(is_player())
        .each_frame(move |entities| {
            for (e, pos) in entities {
                entity::add_component(e, packages::map::components::position(), pos);
            }
        });
}
