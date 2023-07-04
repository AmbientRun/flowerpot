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
use messages::UpdatePlayerAngle;

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
}
