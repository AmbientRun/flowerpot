use ambient_api::{
    core::{
        app::components::main_scene,
        rendering::components::color,
        transform::{
            components::{
                local_to_parent, local_to_world, mesh_to_local, mesh_to_world, spherical_billboard,
                translation,
            },
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use packages::nameplate::{components::*, concepts::*};

#[main]
fn main() {
    spawn_query(nameplate()).bind(move |nameplates| {
        for (e, nameplate) in nameplates {
            create_nameplate(e, nameplate);
        }
    });

    change_query(nameplate())
        .track_change(nameplate())
        .bind(move |nameplates| {
            for (e, nameplate) in nameplates {
                create_nameplate(e, nameplate);
            }
        });

    query((translation(), container(), offset())).each_frame(move |entities| {
        for (_e, (position, container, offset)) in entities {
            let new_translation = position + offset;
            entity::add_component(container, translation(), new_translation);
        }
    });

    despawn_query(container()).bind(move |nameplates| {
        for (_e, container) in nameplates {
            entity::despawn_recursive(container);
        }
    });
}

fn create_nameplate(base: EntityId, (show, _offset): (bool, Vec3)) {
    if let Some(old) = entity::get_component(base, container()) {
        entity::despawn_recursive(old);
    }

    if !show {
        return;
    }

    // TODO: AmbientRun/Ambient #719
    let name = entity::get_component(base, name()).unwrap();

    let approximate_char_width = 36.0;
    let width = name.chars().count() as f32 * approximate_char_width;
    let transform = Mat4::from_scale(Vec3::ONE * 0.005)
        * Mat4::from_rotation_x(180_f32.to_radians())
        * Mat4::from_translation(Vec3::new(-width / 2.0, 0.0, 0.0));

    use ambient_api::core::text::{components::*, types::*};
    let nameplate = Entity::new()
        .with(local_to_parent(), transform)
        .with(text(), name)
        .with(font_size(), 72.0)
        .with(
            font_family(),
            "https://github.com/madmalik/mononoki/raw/main/export/mononoki-Regular.ttf".to_string(),
        )
        .with(font_style(), FontStyle::Regular)
        .with(color(), vec4(1.0, 1.0, 1.0, 1.0))
        .with(main_scene(), ())
        .with(local_to_world(), Mat4::IDENTITY)
        .with(mesh_to_local(), Mat4::IDENTITY)
        .with(mesh_to_world(), Mat4::IDENTITY)
        .spawn();

    let new_container = make_transformable()
        .with(main_scene(), ())
        .with(local_to_world(), Mat4::IDENTITY)
        .with(spherical_billboard(), ())
        .spawn();

    entity::add_child(new_container, nameplate);
    entity::add_component(base, container(), new_container);
}
