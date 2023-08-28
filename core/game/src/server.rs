use ambient_api::prelude::*;

use packages::{
    actions::messages::*,
    crops::components::{
        class, is_medium_crop, is_medium_crop_class, medium_crop_occupant, on_tile,
    },
    items::components::held_ref,
    player::components::{left_hand_ref, right_hand_ref},
    this::components::{pick_up_item_class, place_medium_crop},
};

#[main]
pub fn main() {
    spawn_query(())
        .requires((is_medium_crop_class(), pick_up_item_class()))
        .bind(move |entities| {
            for (e, _) in entities {
                RegisterMediumCropAction::new(
                    "pick_up".to_string(),
                    e,
                    EntityId::null(),
                    EntityId::null(),
                )
                .send_local_broadcast(false);
            }
        });

    spawn_query(())
        .requires(place_medium_crop())
        .bind(move |entities| {
            for (e, _) in entities {
                RegisterTileAction::new("place_medium".to_string(), e, EntityId::null())
                    .send_local_broadcast(false);
            }
        });

    OnAction::subscribe(move |source, data| {
        if source.local().is_none() {
            return;
        }

        if data.id != "pick_up" {
            return;
        }

        let Some(item) = entity::get_component(data.target, pick_up_item_class()) else {
            return;
        };

        let hand = if data.right_is_primary {
            right_hand_ref()
        } else {
            left_hand_ref()
        };

        let hand = entity::get_component(data.player, hand).unwrap();
        entity::add_component(hand, held_ref(), item);

        let crop = data.target;
        let Some(tile) = entity::get_component(crop, on_tile()) else {
            return;
        };

        entity::despawn_recursive(crop);
        entity::add_component(tile, medium_crop_occupant(), EntityId::null());
    });

    OnAction::subscribe(move |source, data| {
        if source.local().is_none() {
            return;
        }

        if data.id != "place_medium" {
            return;
        }

        let hand = if data.right_is_primary {
            right_hand_ref()
        } else {
            left_hand_ref()
        };

        let hand = entity::get_component(data.player, hand).unwrap();

        let Some(item) = entity::get_component(hand, held_ref()) else {
            return;
        };

        let Some(place) = entity::get_component(item, place_medium_crop()) else {
            return;
        };

        let tile = data.target;
        let crop = Entity::new()
            .with(is_medium_crop(), ())
            .with(class(), place)
            .with(on_tile(), tile)
            .spawn();

        entity::add_component(tile, medium_crop_occupant(), crop);
        entity::add_component(hand, held_ref(), EntityId::null());
    });
}
