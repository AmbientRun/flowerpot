use ambient_api::{
    core::player::components::{is_player, user_id},
    prelude::*,
};

use embers::{
    fauna::components::is_fauna,
    map::{
        components::{chunk, in_chunk},
        messages::{LoadPlayerChunk, UnloadPlayerChunk},
    },
    player::{components::*, messages::*},
};

mod shared;

#[main]
fn main() {
    shared::init_shared_player();

    let chunks = flowerpot_common::init_map(chunk());

    spawn_query((is_player(), is_fauna())).bind(move |entities| {
        for (e, _) in entities {
            let left_hand = Entity::new().with(owner_ref(), e).spawn();
            let right_hand = Entity::new().with(owner_ref(), e).spawn();

            entity::add_components(
                e,
                Entity::new()
                    .with_default(is_fauna())
                    .with(speed(), 1.0)
                    .with(position(), vec2(0.0, 0.0))
                    .with(direction(), vec2(0.0, 0.0))
                    .with(yaw(), 0.0)
                    .with(left_hand_ref(), left_hand)
                    .with(right_hand_ref(), right_hand),
            );

            entity::add_components(
                e,
                Entity::new()
                    .with(position(), vec2(0.0, 0.0))
                    .with(loaded_chunks(), vec![]),
            );
        }
    });

    despawn_query((is_player(), user_id(), loaded_chunks())).bind({
        let chunks = chunks.clone();
        move |entities| {
            let chunks = chunks.read().unwrap();
            for (player, (_, uid, loaded)) in entities {
                for position in loaded {
                    let Some(chunk) = chunks.get(&position) else { continue };
                    UnloadPlayerChunk::new(*chunk, position, player, uid.clone())
                        .send_local_broadcast(true);
                }
            }
        }
    });

    change_query((user_id(), loaded_chunks(), in_chunk()))
        .track_change(in_chunk())
        .bind({
            let chunks = chunks.clone();
            move |entities| {
                let chunks = chunks.read().unwrap();
                for (e, (uid, old_chunks, current_chunk)) in entities {
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
                            let Some(chunk) = chunks.get(new_chunk) else { continue };
                            LoadPlayerChunk::new(*chunk, *new_chunk, e, uid.clone())
                                .send_local_broadcast(true);
                        }
                    }

                    for old_chunk in old_chunks.iter() {
                        if !new_chunks.contains(old_chunk) {
                            let Some(chunk) = chunks.get(old_chunk) else { continue };
                            UnloadPlayerChunk::new(*chunk, *old_chunk, e, uid.clone())
                                .send_local_broadcast(true);
                        }
                    }

                    entity::set_component(e, loaded_chunks(), new_chunks);
                }
            }
        });

    UpdatePlayerDirection::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else { return };
        entity::add_component(e, direction(), data.direction.clamp_length_max(1.0));
    });

    UpdatePlayerAngle::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else { return };
        entity::add_component(e, yaw(), data.yaw);
    });

    query(position())
        .requires(is_player())
        .each_frame(move |entities| {
            for (e, pos) in entities {
                entity::add_component(e, position(), pos);
            }
        });
}
