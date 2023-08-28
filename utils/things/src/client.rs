use ambient_api::{core::prefab::components::prefab_from_url, prelude::*};
use flowerpot_common::RemoteEntityStore;

mod shared;

use packages::{
    region_networking::components::remote_entity,
    this::{components::*, messages::*},
};

#[main]
fn main() {
    shared::init_shared();

    let store = RemoteEntityStore::new(remote_entity());

    UpdateThingClass::subscribe(move |_, data| {
        if let Some(e) = store.remote_to_local(data.thing) {
            eprintln!("{:#?}", data);
            entity::add_component(e, class_ref(), data.class);
        }
    });

    spawn_query(model_prefab_url())
        .requires(is_thing())
        .bind(move |entities| {
            for (e, prefab_url) in entities {
                entity::add_component(e, prefab_from_url(), prefab_url);
            }
        });
}
