use ambient_api::prelude::*;

mod shared;

use packages::{
    region_networking::messages::OnSpawnThing,
    this::{components::*, messages::UpdateThingClass},
};

#[main]
fn main() {
    shared::init_shared();

    OnSpawnThing::subscribe(move |source, spawn| {
        // client hack-proof this subscriber
        if source.local().is_none() {
            return;
        }

        if let Some(class) = entity::get_component(spawn.thing, class_ref()) {
            // update the client with the class
            UpdateThingClass::new(spawn.thing, class)
                .send_client_targeted_reliable(spawn.player_uid);
        }
    });
}
