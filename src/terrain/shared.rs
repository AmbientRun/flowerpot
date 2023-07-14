use ambient_api::{glam::DVec2, prelude::*};
use flowerpot::CHUNK_SIZE;
use noise::{Fbm, NoiseFn};

use crate::components::{
    map::{chunk, in_chunk, position},
    terrain::*,
};

pub const RESOLUTION: f64 = 1.0 / 256.0;

pub fn init_shared_terrain() {
    let noise = Fbm::<noise::OpenSimplex>::new(0);

    spawn_query(chunk()).bind(move |entities| {
        for (e, chunk_xy) in entities {
            let heights_dim = CHUNK_SIZE + 1;
            let mut heights = Vec::with_capacity(heights_dim * heights_dim);
            let heights_dim = heights_dim as i32;
            for y in 0..heights_dim {
                for x in 0..heights_dim {
                    let x = x + (CHUNK_SIZE as i32 * chunk_xy.x);
                    let y = y + (CHUNK_SIZE as i32 * chunk_xy.y);
                    let sample = DVec2::new(x as f64, y as f64);
                    let sample = sample * RESOLUTION;
                    let height = noise.get([sample.x, sample.y]);
                    let height = (height * 64.0).round() as i16;
                    heights.push(height);
                }
            }

            entity::add_component(e, heightmap(), heights);
        }
    });

    query((position(), in_chunk())).each_frame(move |entities| {
        for (e, (position, in_chunk)) in entities {
            let Some(chunk_xy) = entity::get_component(in_chunk, chunk()) else { continue };

            let local_pos = position - (chunk_xy * CHUNK_SIZE as i32).as_vec2();
            let coarse_pos = local_pos.floor().as_ivec2();
            let fine_pos = local_pos - coarse_pos.as_vec2();

            if coarse_pos.x < 0
                || coarse_pos.y < 0
                || coarse_pos.x >= CHUNK_SIZE as i32
                || coarse_pos.y >= CHUNK_SIZE as i32
            {
                continue;
            }

            let Some(heights) = entity::get_component(in_chunk, heightmap()) else { continue };

            let get_height = |x, y| {
                let x = x as usize;
                let y = y as usize;
                let idx = y * (CHUNK_SIZE + 1) + x;
                heights[idx] as f32 / 4.0
            };

            let IVec2 { x, y } = coarse_pos;
            let v1 = get_height(x, y);
            let v2 = get_height(x + 1, y);
            let v3 = get_height(x, y + 1);
            let v4 = get_height(x + 1, y + 1);

            let (base, flip_x, dx, flip_y, dy) = if fine_pos.x + fine_pos.y < 1.0 {
                (v1, false, v2 - v1, false, v3 - v1)
            } else {
                (v4, true, v3 - v4, true, v2 - v4)
            };

            let x = if flip_x { 1.0 - fine_pos.x } else { fine_pos.x };
            let y = if flip_y { 1.0 - fine_pos.y } else { fine_pos.y };
            let new_height = base + x * dx + y * dy;
            eprintln!("new height: {}", new_height);
            entity::add_component(e, height(), new_height);
        }
    });
}
