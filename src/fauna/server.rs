use ambient_api::{
    components::core::player::{player, user_id},
    ecs::SupportedValue,
    prelude::*,
};

mod shared;

use components::{fauna::*, map::position};
use messages::*;

#[main]
fn main() {
    let players = query((player(), user_id(), fauna())).build();
    spawn_query(fauna()).bind(move |entities| {
        for (e, _) in entities {
            for (player_entity, (_, player_uid, _)) in players.evaluate() {
                SpawnFauna::new(e).send_client_targeted_reliable(player_uid.clone());
                OnSpawnFauna::new(e, player_entity, player_uid).send_local_broadcast(true);
            }
        }
    });

    let all_fauna = query(fauna()).build();
    spawn_query((player(), user_id(), fauna())).bind(move |entities| {
        for (player_entity, (_, player_uid, _)) in entities {
            for (e, _) in all_fauna.evaluate() {
                SpawnFauna::new(e).send_client_targeted_reliable(player_uid.clone());
                OnSpawnFauna::new(e, player_entity, player_uid.clone()).send_local_broadcast(true);
            }
        }
    });

    despawn_query(fauna()).bind(move |entities| {
        for (e, _) in entities {
            DespawnFauna::new(e).send_client_broadcast_reliable();
        }
    });

    bind_fauna_update(position(), move |e, player, position| {
        UpdateFaunaPosition::new(e, position).send_client_targeted_reliable(player);
    });

    bind_fauna_update(yaw(), move |e, player, yaw| {
        UpdateFaunaYaw::new(e, yaw).send_client_targeted_reliable(player);
    });

    bind_fauna_update(name(), move |e, player, name| {
        eprintln!("updating name: {}", name);
        UpdateFaunaName::new(e, name).send_client_targeted_reliable(player);
    });
}

fn bind_fauna_update<T: Clone + SupportedValue + 'static>(
    component: Component<T>,
    cb: impl Fn(EntityId, String, T) + Clone + 'static,
) {
    let players = query((player(), user_id(), fauna())).build();
    change_query(component)
        .track_change(component)
        .requires(fauna())
        .bind({
            let cb = cb.clone();
            move |entities| {
                for (e, data) in entities {
                    for (_e, (_, player_uid, _)) in players.evaluate() {
                        cb(e, player_uid.clone(), data.clone());
                    }
                }
            }
        });

    OnSpawnFauna::subscribe(move |source, spawn| {
        if source.local().is_none() {
            return;
        }

        if let Some(data) = entity::get_component(spawn.fauna, component) {
            cb(spawn.fauna, spawn.player_uid, data);
        }
    });
}
