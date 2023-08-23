use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::prelude::*;
use flowerpot_common::{ActorExt, SystemExt};

use packages::region_networking::{components::*, messages::*};

#[main]
fn main() {
    let store: Arc<Mutex<HashMap<EntityId, EntityId>>> = Default::default();

    store.on_message(move |store, _, data: SpawnThing| {
        if store.contains_key(&data.thing) {
            return;
        }

        let e = Entity::new().with(remote_entity(), data.thing).spawn();
        store.insert(data.thing, e);
    });

    store.on_message(move |store, _, data: DespawnThing| {
        if let Some(e) = store.get(&data.thing) {
            entity::despawn_recursive(*e);
        }
    });

    store.on_event(despawn_query(remote_entity()), move |store, _e, remote| {
        store.remove(&remote);
    });
}
