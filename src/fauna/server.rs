use ambient_api::prelude::*;

mod shared;

use components::{fauna::*, map::position};
use messages::*;

#[main]
fn main() {
    spawn_query(fauna()).bind(move |entities| {
        for (e, _) in entities {
            SpawnFauna::new(e).send_client_broadcast_reliable();
        }
    });

    despawn_query(fauna()).bind(move |entities| {
        for (e, _) in entities {
            DespawnFauna::new(e).send_client_broadcast_reliable();
        }
    });

    change_query(position())
        .track_change(position())
        .requires(fauna())
        .bind(move |entities| {
            for (e, position) in entities {
                UpdateFaunaPosition::new(e, position).send_client_broadcast_reliable();
            }
        });

    change_query(yaw())
        .track_change(yaw())
        .requires(fauna())
        .bind(move |entities| {
            for (e, yaw) in entities {
                UpdateFaunaYaw::new(e, yaw).send_client_broadcast_reliable();
            }
        });
}
