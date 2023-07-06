use ambient_api::{components::core::player::player, prelude::*};

use crate::components::player::*;

/// Moves the player with the given position by the given delta.
pub fn move_player(position: Vec2, delta: Vec2) -> Vec2 {
    // TODO more advanced movement code using slopes, fences, roads, etc.
    position + delta * delta_time()
}

pub fn init_shared_player() {
    // TODO terrain-based speed
    // TODO sprinting + stamina?
    spawn_query((player(), position())).bind(move |entities| {
        for (e, _) in entities {
            entity::set_component(e, speed(), 3.0);
        }
    });

    // move players
    query((player(), position(), direction(), yaw(), speed())).each_frame(move |entities| {
        for (e, (_player, old_position, direction, yaw, speed)) in entities {
            let rotate = Mat2::from_angle(yaw);
            let delta = rotate * direction * speed;
            let new_position = move_player(old_position, delta);
            entity::set_component(e, position(), new_position);
        }
    });
}
