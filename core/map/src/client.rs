use ambient_api::prelude::*;

use packages::this::components::*;

mod shared;

#[main]
pub fn main() {
    shared::init_shared_map();

    entity::add_component(entity::resources(), is_mod_loaded(), ());
}
