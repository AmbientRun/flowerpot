use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    core::{
        app::components::main_scene,
        camera::components::aspect_ratio_from_window,
        camera::concepts::make_perspective_infinite_reverse_camera,
        player::components::{is_player, local_user_id, user_id},
        transform::{
            components::{local_to_parent, local_to_world, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use embers::{
    fauna::components::{pitch, yaw},
    map::components::position,
    player::{components::*, messages::*},
    terrain::components::altitude,
};

use shared::init_shared_player;

mod shared;

// TODO make a component?
const HEAD_HEIGHT: f32 = 1.5;

#[main]
fn main() {
    spawn_query((is_player(), user_id())).bind(move |players| {
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

            let init_hand = |hand_ref, offset| {
                let hand = Entity::new()
                    .with_default(main_scene())
                    .with_default(local_to_parent())
                    .with_default(local_to_world())
                    .with(translation(), offset)
                    .with(rotation(), Quat::from_rotation_x(-FRAC_PI_2))
                    .with(scale(), Vec3::splat(0.3))
                    .spawn();

                entity::add_child(head, hand);

                entity::add_component(player_entity, hand_ref, hand);
            };

            init_hand(left_hand_ref(), Vec3::new(-0.5, -0.4, 1.0));
            init_hand(right_hand_ref(), Vec3::new(0.5, -0.4, 1.0));

            entity::add_components(
                player_entity,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(local_to_world())
                    .with(position(), Vec2::ZERO)
                    .with(speed(), 10.0) // TODO terrain-based speed
                    .with(head_ref(), head),
            );

            entity::add_component(entity::resources(), local_player_ref(), player_entity);
        }
    });

    change_query((is_player(), yaw(), pitch()))
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

    change_query((is_player(), position(), altitude()))
        .track_change((position(), altitude()))
        .bind(move |entities| {
            for (e, (_, position, altitude)) in entities {
                let new_translation = position.extend(altitude);
                entity::add_component(e, translation(), new_translation);
            }
        });

    change_query((is_player(), pitch(), yaw()))
        .track_change((pitch(), yaw()))
        .bind(move |entities| {
            for (_, (_, pitch, yaw)) in entities {
                UpdatePlayerAngle::new(pitch, yaw).send_server_reliable();
            }
        });

    change_query((is_player(), direction()))
        .track_change(direction())
        .bind(move |entities| {
            for (_, (_, direction)) in entities {
                UpdatePlayerDirection::new(direction).send_server_reliable();
            }
        });
}
