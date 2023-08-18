use ambient_api::prelude::*;

use packages::{
    crops::components::is_medium_crop,
    nameplate::{components::name, concepts::make_nameplate},
};

mod shared;

#[main]
fn main() {
    spawn_query(())
        .requires((is_medium_crop(), name()))
        .bind(move |entities| {
            for (e, _) in entities {
                entity::add_components(e, make_nameplate());
            }
        });
}
