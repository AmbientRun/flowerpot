use ambient_api::{glam::DVec2, prelude::*};
use flowerpot_common::CHUNK_SIZE;
use noise::{Fbm, NoiseFn};

use crate::packages::{
    map::components::{chunk, in_chunk, position},
    this::components::*,
};

pub const RESOLUTION: f64 = 1.0 / 256.0;

pub fn init_shared_terrain() {
    let noise = Fbm::<noise::OpenSimplex>::new(0);

    spawn_query(chunk()).bind(move |entities| {
        for (e, chunk_xy) in entities {
            let altitudes_dim = CHUNK_SIZE + 1;
            let mut altitudes = Vec::with_capacity(altitudes_dim * altitudes_dim);
            let altitudes_dim = altitudes_dim as i32;
            for y in 0..altitudes_dim {
                for x in 0..altitudes_dim {
                    let x = x + (CHUNK_SIZE as i32 * chunk_xy.x);
                    let y = y + (CHUNK_SIZE as i32 * chunk_xy.y);
                    let sample = DVec2::new(x as f64, y as f64);
                    let sample = sample * RESOLUTION;
                    let new_altitude = noise.get([sample.x, sample.y]);
                    let new_altitude = (new_altitude * 64.0).round() as i16;
                    altitudes.push(new_altitude);
                }
            }

            entity::add_component(e, heightmap(), altitudes);
        }
    });

    spawn_query((position(), in_chunk())).bind(move |entities| {
        for (e, (position, in_chunk)) in entities {
            update_altitude(e, position, in_chunk);
        }
    });

    change_query((position(), in_chunk()))
        .track_change(in_chunk())
        .bind(move |entities| {
            for (e, (position, in_chunk)) in entities {
                update_altitude(e, position, in_chunk);
            }
        });
}

pub fn update_altitude(e: EntityId, position: Vec2, in_chunk: EntityId) {
    let Some(chunk_xy) = entity::get_component(in_chunk, chunk()) else {
        return;
    };

    let local_pos = position - (chunk_xy * CHUNK_SIZE as i32).as_vec2();

    let Some(altitudes) = entity::get_component(in_chunk, heightmap()) else {
        return;
    };

    if let Some(new_altitude) = calculate_altitude(local_pos, &altitudes) {
        entity::add_component(e, altitude(), new_altitude);
    }
}

pub fn calculate_altitude(local_pos: Vec2, altitudes: &[i16]) -> Option<f32> {
    let coarse_pos = local_pos.floor().as_ivec2();
    let fine_pos = local_pos - coarse_pos.as_vec2();

    if coarse_pos.x < 0
        || coarse_pos.y < 0
        || coarse_pos.x >= CHUNK_SIZE as i32
        || coarse_pos.y >= CHUNK_SIZE as i32
    {
        return None;
    }

    let get_altitude = |x, y| {
        let x = x as usize;
        let y = y as usize;
        let idx = y * (CHUNK_SIZE + 1) + x;
        altitudes[idx] as f32 / 4.0
    };

    let IVec2 { x, y } = coarse_pos;
    let v1 = get_altitude(x, y);
    let v2 = get_altitude(x + 1, y);
    let v3 = get_altitude(x, y + 1);
    let v4 = get_altitude(x + 1, y + 1);

    let (base, flip_x, dx, flip_y, dy) = if fine_pos.x + fine_pos.y < 1.0 {
        (v1, false, v2 - v1, false, v3 - v1)
    } else {
        (v4, true, v3 - v4, true, v2 - v4)
    };

    let x = if flip_x { 1.0 - fine_pos.x } else { fine_pos.x };
    let y = if flip_y { 1.0 - fine_pos.y } else { fine_pos.y };
    let altitude = base + x * dx + y * dy;

    Some(altitude)
}
