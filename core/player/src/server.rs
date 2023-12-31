use ambient_api::{
    core::player::components::{is_player, user_id},
    prelude::*,
};

use flowerpot_common::SystemExt;
use packages::{
    fauna::components::{is_fauna, yaw},
    map::components::{chunk, in_chunk, position},
    region_networking::messages::{LoadPlayerRegion, UnloadPlayerRegion},
    things::components::{class_ref, is_class, model_prefab_url},
    this::{assets::url, components::*, messages::*},
};
use shared::*;

mod shared;

#[main]
fn main() {
    let player_class = Entity::new()
        .with(is_class(), ())
        .with(is_fauna(), ())
        .with(speed(), 30.0)
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
                    .with(loaded_chunks(), vec![])
                    .with(chunk_sequence(), 1)
                    .with(input_sequence(), 0),
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
        change_query((user_id(), chunk_sequence(), loaded_chunks(), in_chunk()))
            .track_change(in_chunk()),
        move |chunks, e, (uid, old_sequence, old_chunks, current_chunk)| {
            let current_pos = entity::get_component(current_chunk, chunk()).unwrap();
            let mut new_chunks = Vec::new();
            for y in -4..4 {
                for x in -4..4 {
                    new_chunks.push(ivec2(x, y) + current_pos);
                }
            }

            if new_chunks == old_chunks {
                return;
            }

            for new in new_chunks.iter() {
                if !old_chunks.contains(new) {
                    let Some(chunk) = chunks.get(new) else {
                        continue;
                    };

                    LoadPlayerRegion::new(*chunk, e, uid.clone()).send_local_broadcast(false);
                }
            }

            for old in old_chunks.iter() {
                if !new_chunks.contains(old) {
                    let Some(chunk) = chunks.get(old) else {
                        continue;
                    };

                    UnloadPlayerRegion::new(*chunk, e, uid.clone()).send_local_broadcast(false);
                }
            }

            UpdateLoadedChunks::new(new_chunks.clone(), old_sequence)
                .send_client_targeted_reliable(uid);

            entity::set_component(e, loaded_chunks(), new_chunks);
            entity::set_component(e, chunk_sequence(), old_sequence + 1);
        },
    );

    UpdatePlayerInput::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else {
            return;
        };

        let Some(old_sequence) = entity::get_component(e, input_sequence()) else {
            // println!("player has no input sequence");
            return;
        };

        // ignore out-of-order input updates
        if old_sequence >= data.sequence {
            // println!("out-of-order input sequence: {}", data.sequence);
            return;
        }

        let state = InputState {
            direction: data.direction.clamp_length_max(1.0),
            pitch: data.pitch,
            yaw: data.yaw,
        };

        state.set(e);
        entity::set_component(e, input_sequence(), data.sequence);
    });

    query((input_sequence(), user_id()))
        .requires(is_player())
        .each_frame(move |entities| {
            let dt = delta_time();
            for (e, (sequence, uid)) in entities {
                let Some(state) = update_player(e, dt) else {
                    continue;
                };

                UpdatePlayerState {
                    position: state.position,
                    speed: state.speed,
                    sequence,
                }
                .send_client_targeted_unreliable(uid);
            }
        });

    query(position())
        .requires(is_player())
        .each_frame(move |entities| {
            for (e, pos) in entities {
                entity::add_component(e, packages::map::components::position(), pos);
            }
        });
}
