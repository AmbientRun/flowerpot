use ambient_api::{components::core::player::player, prelude::*};

use components::{fauna, player::*};
use messages::{Join, UpdatePlayerAngle, UpdatePlayerDirection};

mod shared;

#[main]
fn main() {
    Join::subscribe(move |source, _data| {
        let Some(e) = source.client_entity_id() else { return };

        // player component must be attached before fauna spawn messages will
        // be received
        run_async(async move {
            entity::wait_for_component(e, player()).await;

            if entity::has_component(e, fauna::fauna()) {
                return;
            }

            entity::add_components(
                e,
                Entity::new()
                    .with_default(fauna::fauna())
                    .with(speed(), 1.0)
                    .with(position(), vec2(0.0, 0.0))
                    .with(direction(), vec2(0.0, 0.0))
                    .with(yaw(), 0.0),
            );

            entity::add_component(e, position(), vec2(0.0, 0.0));
        });
    });

    UpdatePlayerDirection::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else { return };
        entity::add_component(e, direction(), data.direction.clamp_length_max(1.0));
    });

    UpdatePlayerAngle::subscribe(move |source, data| {
        let Some(e) = source.client_entity_id() else { return };
        entity::add_component(e, yaw(), data.yaw);
    });

    shared::init_shared_player();

    query(position())
        .requires(player())
        .each_frame(move |entities| {
            for (e, pos) in entities {
                entity::add_component(e, fauna::position(), pos);
            }
        });
}
