use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::{local_user_id, player, user_id},
        primitives::cube,
        transform::{local_to_parent, rotation, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    messages::Frame,
    prelude::*,
};

use components::player::*;
use messages::{UpdatePlayerAngle, UpdatePlayerDirection};
use shared::init_shared_player;

mod shared;

// TODO make a component?
const HEAD_HEIGHT: f32 = 1.5;

#[main]
fn main() {
    spawn_query((player(), user_id())).bind(move |players| {
        let local_user_id = entity::get_component(entity::resources(), local_user_id()).unwrap();
        for (player_entity, (_, user)) in players {
            if user != local_user_id {
                continue;
            }

            let head = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with_default(main_scene())
                .with(user_id(), user.clone())
                .with(translation(), Vec3::Z * HEAD_HEIGHT)
                .with_default(local_to_parent())
                .with(rotation(), Quat::from_rotation_x(FRAC_PI_2))
                .spawn();

            entity::add_child(player_entity, head);

            entity::add_components(
                player_entity,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(position(), Vec2::ZERO)
                    .with(speed(), 10.0) // TODO terrain-based speed
                    .with(head_ref(), head),
            );

            entity::add_component(entity::resources(), local_player_ref(), player_entity);
        }
    });

    change_query((player(), yaw(), pitch()))
        .track_change((yaw(), pitch()))
        .bind(move |players| {
            for (e, (_player, yaw, pitch)) in players {
                entity::add_component(e, rotation(), Quat::from_rotation_z(yaw));
                if let Some(head) = entity::get_component(e, head_ref()) {
                    entity::add_component(
                        head,
                        rotation(),
                        Quat::from_rotation_x(FRAC_PI_2 + pitch),
                    );
                }
            }
        });

    init_shared_player();

    change_query((player(), position()))
        .track_change(position())
        .bind(move |entities| {
            for (e, (_, position)) in entities {
                // TODO integrate with map system
                let new_translation = position.extend(0.0);
                entity::add_component(e, translation(), new_translation);
            }
        });

    run_async(async_main());
}

async fn async_main() {
    let local_player_entity = entity::get_component(entity::resources(), local_player_ref())
        .expect("local_player_ref resource was deleted");

    Frame::subscribe({
        use pitch as pitch_component;
        use yaw as yaw_component;

        let mut cursor_lock = input::CursorLockGuard::new(true);
        let mut pitch = 0.0;
        let mut yaw = 0.0;

        move |_| {
            let input = input::get();
            // TODO make cursor lock component-based for easier extension
            if !cursor_lock.auto_unlock_on_escape(&input) {
                return;
            }

            let pitch_factor = 0.01;
            let yaw_factor = 0.01;
            yaw = (yaw + input.mouse_delta.x * yaw_factor) % TAU;
            pitch = (pitch + input.mouse_delta.y * pitch_factor).clamp(-FRAC_PI_2, FRAC_PI_2);

            UpdatePlayerAngle::new(pitch, yaw).send_server_reliable();

            entity::add_component(local_player_entity, yaw_component(), yaw);
            entity::add_component(local_player_entity, pitch_component(), pitch);
        }
    });

    Frame::subscribe({
        move |_| {
            let input = input::get();
            // TODO interop with mouselook cursor lock?

            let mut new_direction = Vec2::ZERO;
            if input.keys.contains(&KeyCode::W) {
                new_direction.y -= 1.0;
            }
            if input.keys.contains(&KeyCode::S) {
                new_direction.y += 1.0;
            }
            if input.keys.contains(&KeyCode::A) {
                new_direction.x -= 1.0;
            }
            if input.keys.contains(&KeyCode::D) {
                new_direction.x += 1.0;
            }

            let new_direction = new_direction.clamp_length_max(1.0);
            entity::add_component(local_player_entity, direction(), new_direction);
            UpdatePlayerDirection::new(new_direction).send_server_reliable();
        }
    });
}
