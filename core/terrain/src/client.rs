use ambient_api::{
    core::{
        procedurals::components::procedural_mesh,
        rendering::components::{color, pbr_material_from_url},
        transform::concepts::make_transformable,
    },
    mesh::{self, Vertex},
    prelude::*,
};
use flowerpot_common::CHUNK_SIZE;

use embers::{
    map::components::chunk,
    terrain::{assets, components::heightmap},
};

mod shared;

#[main]
fn main() {
    shared::init_shared_terrain();

    spawn_query((chunk(), heightmap())).bind(move |entities| {
        for (e, (chunk_xy, altitudes)) in entities {
            let vertex_num = CHUNK_SIZE * CHUNK_SIZE * 6;
            let altitudes_dim = CHUNK_SIZE + 1;

            let mut vertices = Vec::with_capacity(vertex_num);
            let mut indices = Vec::with_capacity(vertex_num);

            let v_pos = |v: (usize, usize)| {
                let (x, y) = v;
                let idx = y * altitudes_dim + x;
                let altitude = altitudes[idx];
                let z = altitude as f32 / 4.0;
                let chunk_offset = chunk_xy * (CHUNK_SIZE as i32);
                let xy = vec2(x as f32, y as f32) + chunk_offset.as_vec2();
                xy.extend(z)
            };

            let mut add_tri = |v1, v2, v3| {
                let (v1, uv1) = v1;
                let (v2, uv2) = v2;
                let (v3, uv3) = v3;

                let v1 = v_pos(v1);
                let v2 = v_pos(v2);
                let v3 = v_pos(v3);

                let normal = (v2 - v1).cross(v3 - v1).normalize();

                // TODO fill these out
                let tangent = Vec3::ONE;

                let v = |position, texcoord0| Vertex {
                    position,
                    normal,
                    tangent,
                    texcoord0,
                };

                let voff = vertices.len() as u32;
                vertices.push(v(v1, uv1));
                vertices.push(v(v2, uv2));
                vertices.push(v(v3, uv3));

                indices.extend([voff, voff + 1, voff + 2]);
            };

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let v1 = ((x, y), vec2(0.0, 0.0));
                    let v2 = ((x + 1, y), vec2(1.0, 0.0));
                    let v3 = ((x, y + 1), vec2(0.0, 1.0));
                    let v4 = ((x + 1, y + 1), vec2(1.0, 1.0));
                    add_tri(v1, v2, v3);
                    add_tri(v3, v2, v4);
                }
            }

            let mesh = mesh::create(&mesh::Descriptor {
                vertices: vertices.as_ref(),
                indices: indices.as_ref(),
            });

            let mesh = Entity::new()
                .with_merge(make_transformable())
                .with(procedural_mesh(), mesh)
                .with(
                    pbr_material_from_url(),
                    assets::url("assets/pipeline.toml/0/mat.json"),
                )
                .with(color(), Vec4::ONE)
                .spawn();

            entity::add_child(e, mesh);
        }
    });
}
