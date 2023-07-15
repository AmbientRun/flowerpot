use ambient_api::{
    components::core::player::{player, user_id},
    prelude::*,
};

use components::{fauna, map, player::*};
use messages::{
    Join, OnPlayerLoadChunk, OnPlayerUnloadChunk, UpdatePlayerAngle, UpdatePlayerDirection,
};

mod shared;

#[main]
fn main() {
    shared::init_shared_player();

    let all_chunks = query(map::chunk()).build();
    Join::subscribe(move |source, _data| {
        let Some(e) = source.client_entity_id() else { return };
        let Some(uid)  = source.client_user_id() else { return };

        // player component must be attached before fauna spawn messages will
        // be received
        run_async(async move {
            entity::wait_for_component(entity::resources(), map::mod_loaded()).await;
            entity::wait_for_component(e, player()).await;

            // deduplicate already-joined players
            if entity::has_component(e, fauna::fauna()) {
                return;
            }

            entity::add_components(
                e,
                Entity::new()
                    .with_default(fauna::fauna())
                    .with(speed(), 1.0)
                    .with(position(), vec2(0.0, 0.0))
                    .with(direction(), vec2(0.0, 0.0))
                    .with(yaw(), 0.0),
            );

            entity::add_component(e, position(), vec2(0.0, 0.0));

            for (chunk, position) in all_chunks.evaluate() {
                OnPlayerLoadChunk::new(chunk, position, e, uid.clone()).send_local_broadcast(true);
            }
        });
    });

    let all_chunks = query(map::chunk()).build();
    despawn_query((player(), user_id())).bind(move |entities| {
        for (player, (_, uid)) in entities {
            for (chunk, position) in all_chunks.evaluate() {
                OnPlayerUnloadChunk::new(chunk, position, player, uid.clone())
                    .send_local_broadcast(true);
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
        .requires(player())
        .each_frame(move |entities| {
            for (e, pos) in entities {
                entity::add_component(e, map::position(), pos);
            }
        });
}
