use ambient_api::{
    core::prefab::components::{prefab_from_url, spawned},
    prelude::*,
};
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

    let (prefab_tx, prefab_rx) = flume::unbounded();

    spawn_query(())
        .requires((model_prefab_url(), is_thing()))
        .bind(move |entities| {
            for (e, _) in entities {
                let _ = prefab_tx.send(e);
            }
        });

    run_async(async move {
        while let Ok(e) = prefab_rx.recv_async().await {
            if !entity::exists(e) {
                continue;
            }

            let Some(prefab_url) = entity::get_component(e, model_prefab_url()) else {
                continue;
            };

            entity::add_component(e, prefab_from_url(), prefab_url);
            entity::wait_for_component(e, spawned()).await;
        }
    });
}
