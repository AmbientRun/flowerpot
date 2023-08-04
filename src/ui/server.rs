use ambient_api::prelude::*;

mod shared;

use components::fauna;
use messages::{AcceptJoin, JoinDenied, JoinRequest};

#[main]
fn main() {
    JoinRequest::subscribe(move |source, data| {
        let Some(player_entity) = source.client_entity_id() else { return };
        let Some(uid) = source.client_user_id() else { return };
        AcceptJoin::new().send_client_targeted_reliable(uid);
        entity::add_component(player_entity, fauna::fauna(), ());
        entity::add_component(player_entity, fauna::name(), data.name);
    });
}
