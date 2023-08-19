use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    core::{
        app::components::main_scene,
        camera::components::aspect_ratio_from_window,
        camera::concepts::make_perspective_infinite_reverse_camera,
        player::components::{is_player, local_user_id, user_id},
        primitives::{components::sphere_radius, concepts::make_sphere},
        rendering::components::color,
        transform::{
            components::{local_to_parent, local_to_world, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use packages::{
    fauna::components::{pitch, yaw},
    map::components::{chunk_tile_index, in_chunk, position},
    player::{components::*, messages::*},
    terrain::{
        components::{altitude, highlight_tile},
        messages::{RaycastRequest, RaycastResponse},
    },
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
                .with(main_scene(), ())
                .with(user_id(), user.clone())
                .with(translation(), Vec3::Z * HEAD_HEIGHT)
                .with(local_to_parent(), Mat4::IDENTITY)
                .with(rotation(), Quat::from_rotation_x(FRAC_PI_2))
                .spawn();

            entity::add_child(player_entity, head);

            let init_hand = |hand_ref, offset| {
                let hand = Entity::new()
                    .with(main_scene(), ())
                    .with(local_to_parent(), Mat4::IDENTITY)
                    .with(local_to_world(), Mat4::IDENTITY)
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
                    .with(local_to_world(), Mat4::IDENTITY)
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

    query((yaw(), pitch(), position(), altitude()))
        .requires(is_player())
        .each_frame(move |players| {
            for (_e, (yaw, pitch, position, altitude)) in players {
                let origin = position.extend(altitude + 1.5); // TODO player height component
                let delta = Quat::from_rotation_z(yaw) * Quat::from_rotation_x(pitch) * -Vec3::Y;
                let limit = 10.0; // TODO player reach

                RaycastRequest::new(origin, delta, limit).send_local_broadcast(false);
            }
        });

    let mut last_tile: Option<(EntityId, u8)> = None;
    RaycastResponse::subscribe(move |_, data| {
        if let Some((last_chunk, last_tile_idx)) = last_tile {
            if last_chunk != data.chunk_entity || last_tile_idx != data.tile_idx {
                if let Some(last_highlight) =
                    entity::get_component(player::get_local(), tile_selection_ref())
                {
                    entity::despawn_recursive(last_highlight);
                }

                entity::remove_component(player::get_local(), tile_selection_ref());
                last_tile = None;
            } else {
                // last tile is the currently-highlighted one, so skip
                // highlighting a new one
                return;
            }
        }

        if data.distance < 0.0 {
            return;
        }

        let highlight = Entity::new()
            .with(in_chunk(), data.chunk_entity)
            .with(chunk_tile_index(), data.tile_idx)
            .with(highlight_tile(), ())
            .with(
                ambient_api::core::app::components::name(),
                "Highlighted Tile".into(),
            )
            .spawn();

        last_tile = Some((data.chunk_entity, data.tile_idx));
        entity::add_component(player::get_local(), tile_selection_ref(), highlight);
    });
}
