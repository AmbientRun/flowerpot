use ambient_api::{glam::DVec2, prelude::*};
use noise::{Fbm, NoiseFn};

use crate::components::{map::chunk, terrain::*};

// TODO deduplicate this
pub const CHUNK_SIZE: usize = 16;

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
                    let height = (height * 64.0).round() as u16;
                    heights.push(height);
                }
            }

            entity::add_component(e, heightmap(), heights);
        }
    });
}
