use ambient_api::prelude::*;

use components::map::*;
use messages::{LoadChunk, UnloadChunk};

mod shared;

#[main]
pub fn main() {
    let chunks = shared::init_shared_map();

    LoadChunk::subscribe(move |_, data| {
        println!("Loading chunk: {}", data.pos);
        Entity::new().with(chunk(), data.pos).spawn();
    });

    UnloadChunk::subscribe({
        let chunks = chunks.clone();
        move |_, data| {
            println!("Unloading chunk: {}", data.pos);
            let Some(chunk) = chunks.write().unwrap().remove(&data.pos) else { return };
            entity::despawn_recursive(chunk);
        }
    });

    entity::add_component(entity::resources(), mod_loaded(), ());
}
