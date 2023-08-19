use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
use itertools::izip;
use rapier3d::{
    geometry::TriMesh,
    parry::query::{Ray, RayCast},
};

use packages::{
    map::components::{chunk, chunk_tile_index, in_chunk},
    terrain::{assets, components::*, messages::*},
};

use crate::packages::terrain::messages::RaycastResponse;

mod shared;

type ChunkMeshes = Arc<Mutex<HashMap<IVec2, (EntityId, TriMesh)>>>;

#[main]
fn main() {
    shared::init_shared_terrain();

    let meshes = ChunkMeshes::default();

    spawn_query((chunk(), heightmap())).bind(move |entities| {
        for (e, (chunk_xy, altitudes)) in entities {
            let vertex_num = CHUNK_SIZE * CHUNK_SIZE * 6;

            let mut positions = Vec::with_capacity(vertex_num);
            let mut normals = Vec::with_capacity(vertex_num);
            let mut tangents = Vec::with_capacity(vertex_num);
            let mut texcoords = Vec::with_capacity(vertex_num);
            let mut indices = Vec::with_capacity(vertex_num);

            let v_pos = |v: (usize, usize)| {
                let altitudes_dim = CHUNK_SIZE + 1;
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
                let voff = positions.len() as u32;

                let mut push_v = |position, texcoord| {
                    positions.push(position);
                    normals.push(normal);
                    tangents.push(tangent);
                    texcoords.push(texcoord);
                };

                push_v(v1, uv1);
                push_v(v2, uv2);
                push_v(v3, uv3);

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

            entity::add_components(
                e,
                Entity::new()
                    .with(mesh_positions(), positions)
                    .with(mesh_normals(), normals)
                    .with(mesh_tangents(), tangents)
                    .with(mesh_texcoords(), texcoords)
                    .with(mesh_indices(), indices),
            );
        }
    });

    spawn_query((
        mesh_positions(),
        mesh_normals(),
        mesh_tangents(),
        mesh_texcoords(),
        mesh_indices(),
    ))
    .bind(move |entities| {
        for (e, (positions, normals, tangents, texcoords, indices)) in entities {
            let vertices: Vec<_> = izip!(positions, normals, tangents, texcoords)
                .map(|(position, normal, tangent, texcoord0)| Vertex {
                    position,
                    normal,
                    tangent,
                    texcoord0,
                })
                .collect();

            let mesh = mesh::create(&mesh::Descriptor {
                vertices: vertices.as_ref(),
                indices: indices.as_ref(),
            });

            let mesh = Entity::new()
                .with_merge(make_transformable())
                .with(procedural_mesh(), mesh)
                .with(
                    pbr_material_from_url(),
                    assets::url("pipeline.toml/0/mat.json"),
                )
                .with(color(), Vec4::ONE)
                .spawn();

            entity::add_child(e, mesh);
        }
    });

    spawn_query((chunk(), mesh_positions(), mesh_indices())).bind({
        let meshes = meshes.clone();
        move |entities| {
            let mut meshes = meshes.lock().unwrap();
            for (e, (chunk_xy, positions, indices)) in entities {
                let positions: Vec<_> =
                    positions.into_iter().map(|v| v.to_array().into()).collect();

                let tris: Vec<_> = indices
                    .chunks_exact(3)
                    .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                    .collect();

                let mesh = TriMesh::new(positions, tris);
                meshes.insert(chunk_xy, (e, mesh));
            }
        }
    });

    despawn_query(chunk()).bind({
        let meshes = meshes.clone();
        move |entities| {
            let mut meshes = meshes.lock().unwrap();
            for (_e, xy) in entities {
                meshes.remove(&xy);
            }
        }
    });

    RaycastRequest::subscribe({
        let meshes = meshes.clone();
        move |source, data| {
            let Some(reply) = source.local() else { return };

            // eprintln!("{:#?}", data);

            let ray = Ray::new(data.origin.to_array().into(), data.delta.to_array().into());

            let mut current_chunk = (data.origin.xy() / Vec2::splat(CHUNK_SIZE as f32))
                .floor()
                .as_ivec2();
            let mut local_origin = data.origin.xy() - current_chunk.as_vec2() * CHUNK_SIZE as f32;
            let mut remaining_limit = data.limit;
            while remaining_limit > 0.0 {
                // eprintln!("{remaining_limit}: {current_chunk}@{local_origin}");

                if let Some((chunk_entity, chunk_mesh)) = meshes.lock().unwrap().get(&current_chunk)
                {
                    if let Some(distance) = chunk_mesh.cast_local_ray(&ray, data.limit, false) {
                        let collision_pos = data.origin + data.delta * distance;
                        let tile_pos =
                            collision_pos.xy() - current_chunk.as_vec2() * CHUNK_SIZE as f32;
                        let tile_pos = tile_pos.floor().as_ivec2();
                        let tile_idx = tile_pos.y * CHUNK_SIZE as i32 + tile_pos.x;

                        RaycastResponse {
                            collision_pos,
                            chunk_entity: *chunk_entity,
                            chunk_pos: current_chunk,
                            tile_idx: tile_idx.try_into().unwrap(),
                            distance,
                        }
                        .send_local(reply);

                        return;
                    }
                }

                let edges = [
                    (ivec2(-1, 0), -local_origin.x / data.delta.x),
                    (ivec2(0, -1), -local_origin.y / data.delta.y),
                    (
                        ivec2(1, 0),
                        (CHUNK_SIZE as f32 - local_origin.x) / data.delta.x,
                    ),
                    (
                        ivec2(0, 1),
                        (CHUNK_SIZE as f32 - local_origin.y) / data.delta.y,
                    ),
                ];

                // eprintln!("{:#?}", edges);

                let mut nearest_edge = None;
                for edge in edges {
                    if edge.1 > 0.0 {
                        if let Some((_, nearest_dist)) = nearest_edge {
                            if edge.1 < nearest_dist {
                                nearest_edge = Some(edge);
                            }
                        } else {
                            nearest_edge = Some(edge);
                        }
                    }
                }

                let Some((next_chunk, distance)) = nearest_edge else { break };
                current_chunk += next_chunk;
                local_origin +=
                    data.delta.xy() * distance - next_chunk.as_vec2() * CHUNK_SIZE as f32;
                remaining_limit -= distance;
            }

            RaycastResponse {
                collision_pos: Vec3::ZERO,
                chunk_entity: EntityId::null(),
                chunk_pos: ivec2(0, 0),
                tile_idx: 0,
                distance: -1.0,
            }
            .send_local(reply);
        }
    });

    spawn_query((in_chunk(), chunk_tile_index()))
    .requires(highlight_tile())
    .bind(move |entities| {
        for (e, (chunk_entity, tile)) in entities {
            // TODO batch concept component fetches?
            let Some(positions) = entity::get_component(chunk_entity, mesh_positions()) else { continue };
            let Some(normals) = entity::get_component(chunk_entity, mesh_normals()) else { continue };
            let Some(tangents) = entity::get_component(chunk_entity, mesh_tangents()) else { continue };
            let Some(texcoords) = entity::get_component(chunk_entity, mesh_texcoords()) else { continue };

            let vertices: Vec<_> = izip!(positions, normals, tangents, texcoords)
                // https://stackoverflow.com/questions/32690678/is-it-more-efficient-to-slice-an-array-or-use-iteratorskip
                // alternative is taking slices, which isn't as clean.
                .skip(tile as usize * 6)
                .take(6)
                .map(|(position, normal, tangent, texcoord0)| Vertex {
                    position: position + Vec3::Z * 0.01, // avoid z-fighting
                    normal,
                    tangent,
                    texcoord0,
                })
                .collect();

            let mesh = mesh::create(&mesh::Descriptor {
                vertices: vertices.as_ref(),
                indices: &[0, 1, 2, 3, 4, 5],
            });

            entity::add_components(
                e,
                make_transformable()
                    .with(procedural_mesh(), mesh)
                    .with(
                        pbr_material_from_url(),
                        assets::url("pipeline.toml/1/mat.json"),
                    )
                    .with(color(), Vec4::ONE),
            );
        }
    });
}
