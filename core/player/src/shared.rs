use ambient_api::{core::player::components::is_player, prelude::*};

use crate::packages::{fauna::components::yaw, map::components::position, this::components::*};

/// Moves the player with the given position by the given delta.
pub fn move_player(position: Vec2, delta: Vec2) -> Vec2 {
    // TODO more advanced movement code using slopes, fences, roads, etc.
    position + delta * delta_time()
}

pub fn init_shared_player() {
    // TODO terrain-based speed
    // TODO sprinting + stamina?
    spawn_query((is_player(), position())).bind(move |entities| {
        for (e, _) in entities {
            entity::set_component(e, speed(), 30.0);
        }
    });

    // move players
    query((is_player(), position(), direction(), yaw(), speed())).each_frame(move |entities| {
        for (e, (_player, old_position, direction, yaw, speed)) in entities {
            let rotate = Mat2::from_angle(yaw);
            let delta = rotate * direction * speed;
            let new_position = move_player(old_position, delta);
            entity::set_component(e, position(), new_position);
        }
    });
}
