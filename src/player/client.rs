use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::{local_user_id, player, user_id},
        primitives::cube,
        transform::{local_to_parent, rotation, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

use components::player::*;

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
}
