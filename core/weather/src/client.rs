use std::f64::consts::TAU;

use ambient_api::{
    core::messages::Frame,
    core::{
        app::components::main_scene,
        rendering::components::{light_diffuse, sky, sun},
        transform::{components::rotation, concepts::make_transformable},
    },
    prelude::*,
};

use crate::packages::this::components::*;

mod shared;

#[main]
fn main() {
    let sun_entity = make_transformable()
        .with(sun(), 1.0)
        .with(light_diffuse(), Vec3::ONE * 5.0)
        .with(main_scene(), ())
        .spawn();

    make_transformable().with(sky(), ()).spawn();

    Frame::subscribe(move |_| {
        let to_game_time =
            entity::get_component(entity::synchronized_resources(), real_time_to_game_time())
                .unwrap_or(0.0);

        let tick = delta_time() as f64;
        let hours = tick / 60.0 / 60.0;
        let elapsed = hours * to_game_time;

        let time = entity::mutate_component_with_default(
            entity::synchronized_resources(),
            time_of_day(),
            12.0,
            |time| {
                *time += elapsed;
                while *time > 24.0 {
                    *time -= 24.0;
                }
            },
        );

        let theta = (time / 24.0) * TAU;
        let new_rotation = Quat::from_rotation_y(theta as f32);
        entity::set_component(sun_entity, rotation(), new_rotation);
    });
}
