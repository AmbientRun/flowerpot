use ambient_api::{core::player::components::user_id, ecs::SupportedValue, prelude::*};

mod shared;

use packages::{
    map::components::{in_chunk, position},
    region_networking::{components::players_observing, messages::OnSpawnThing},
    this::{components::*, messages::*},
};

#[main]
fn main() {
    bind_fauna_update(position(), move |e, player, position| {
        UpdateFaunaPosition::new(e, position).send_client_targeted_reliable(player);
    });

    bind_fauna_update(pitch(), move |e, player, pitch| {
        UpdateFaunaPitch::new(e, pitch).send_client_targeted_reliable(player);
    });

    bind_fauna_update(yaw(), move |e, player, yaw| {
        UpdateFaunaYaw::new(e, yaw).send_client_targeted_reliable(player);
    });

    bind_fauna_update(name(), move |e, player, name| {
        eprintln!("updating name: {}", name);
        UpdateFaunaName::new(e, name).send_client_targeted_reliable(player);
    });
}

fn bind_fauna_update<T: Clone + SupportedValue + 'static>(
    component: Component<T>,
    cb: impl Fn(EntityId, String, T) + Clone + 'static,
) {
    change_query((in_chunk(), component))
        .track_change(component)
        .requires(is_fauna())
        .bind({
            let cb = cb.clone();
            move |entities| {
                for (e, (chunk, data)) in entities {
                    for player in
                        entity::get_component(chunk, players_observing()).unwrap_or_default()
                    {
                        let Some(uid) = entity::get_component(player, user_id()) else {
                            continue;
                        };
                        cb(e, uid, data.clone());
                    }
                }
            }
        });

    OnSpawnThing::subscribe(move |source, spawn| {
        if source.local().is_none() {
            return;
        }

        if let Some(data) = entity::get_component(spawn.thing, component) {
            cb(spawn.thing, spawn.player_uid, data);
        }
    });
}
