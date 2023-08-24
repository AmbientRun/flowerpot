use std::collections::HashSet;

use ambient_api::{core::player::components::user_id, prelude::*};

use packages::{
    this::{components::*, messages::*},
    player::components::{left_hand_ref, owner_ref, right_hand_ref},
};

mod shared;

fn update_player_held(e: EntityId) -> Option<()> {
    let uid = entity::get_component(e, user_id())?;
    let left = entity::get_component(e, left_hand_ref())?;
    let right = entity::get_component(e, right_hand_ref())?;

    let left_held = entity::get_component(left, held_ref()).unwrap_or_default();
    let right_held = entity::get_component(right, held_ref()).unwrap_or_default();
    UpdateHeldItems::new(left_held, right_held).send_client_targeted_reliable(uid);

    Some(())
}

#[main]
fn main() {
    change_query((held_ref(), owner_ref()))
        .track_change(held_ref())
        .bind(move |entities| {
            let mut dirty_players = HashSet::new();
            for (_, (_held, owner)) in entities {
                dirty_players.insert(owner);
            }

            for player in dirty_players {
                update_player_held(player);
            }
        });
}
