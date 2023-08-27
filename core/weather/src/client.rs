use std::f64::consts::TAU;

use ambient_api::{
    core::messages::Frame,
    core::{
        app::components::main_scene,
        rendering::components::{light_ambient, light_diffuse, sky, sun},
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

        let theta = (time + 6.0) / 24.0 * TAU;
        let new_rotation = Quat::from_rotation_y(theta as f32);

        let mix_day = cycle_mix(time as f32, 7.0, 17.0, 1.5);
        let mix_night = 1.0 - cycle_mix(time as f32, 7.0, 17.0, 2.0);

        let diffuse_night = Vec3::ZERO;
        let diffuse_day = Vec3::ONE * 5.0;
        let ambient_night = vec3(0.1, 0.2, 0.3) * 0.5;
        let ambient_day = Vec3::ONE * 0.2;

        let diffuse = mix_day * diffuse_day + mix_night * diffuse_night;
        let ambient = mix_day * ambient_day + mix_night * ambient_night;

        entity::add_components(
            sun_entity,
            Entity::new()
                .with(rotation(), new_rotation)
                .with(light_diffuse(), diffuse)
                .with(light_ambient(), ambient),
        );
    });
}

fn cycle_mix(time: f32, sunrise_end: f32, sunset_begin: f32, twilight_length: f32) -> f32 {
    let sunrise_begin = sunrise_end - twilight_length;
    let sunset_end = sunset_begin + twilight_length;

    if time > sunrise_begin && time < sunrise_end {
        (time - sunrise_begin) / twilight_length
    } else if time > sunset_begin && time < sunset_end {
        (time - sunset_end) / -twilight_length
    } else if time > sunrise_end && time < sunset_begin {
        1.0
    } else {
        0.0
    }
}
