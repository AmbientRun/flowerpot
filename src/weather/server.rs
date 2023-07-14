use ambient_api::{
    components::core::{
        app::main_scene,
        rendering::{light_diffuse, sky, sun},
        transform::rotation,
    },
    concepts::make_transformable,
    prelude::*,
};

mod shared;

#[main]
fn main() {
    make_transformable()
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-45_f32.to_radians()))
        .with(light_diffuse(), Vec3::ONE * 5.0)
        .with_default(main_scene())
        .spawn();

    make_transformable().with_default(sky()).spawn();
}
