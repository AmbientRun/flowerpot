use ambient_api::{
    core::{
        ecs::components::children, primitives::components::cube, rendering::components::color,
        transform::components::local_to_parent,
    },
    prelude::*,
};

use embers::{
    items::{components::*, messages::*},
    player::components::{left_hand_ref, local_player_ref, right_hand_ref},
};

mod shared;

#[main]
fn main() {
    run_async(async_main());
}

async fn async_main() {
    let local_player_entity = entity::wait_for_component(entity::resources(), local_player_ref())
        .await
        .expect("local_player_ref resource was deleted");

    let Some(left_hand) = entity::get_component(local_player_entity, left_hand_ref()) else { panic!("local player entity has no left hand") };
    let Some(right_hand) = entity::get_component(local_player_entity, right_hand_ref()) else { panic!("local player entity has no right hand") };

    UpdateHeldItems::subscribe(move |_, data| {
        update_held_item(left_hand, data.left);
        update_held_item(right_hand, data.right);
    });
}

fn update_held_item(hand: EntityId, class: EntityId) {
    for child in entity::get_component(hand, children()).unwrap_or_default() {
        entity::despawn_recursive(child);
    }

    if class.is_null() {
        return;
    }

    let mut item_instance = Entity::new().with_default(local_to_parent());

    if let Some(new_color) = entity::get_component(class, color()) {
        item_instance.set(color(), new_color);
    }

    // if let Some(prefab) = entity::get_component(class, items::prefab_path()) {
    //     item_instance.set(prefab_from_url(), asset::url(prefab).unwrap());
    // } else {
    item_instance.set(cube(), ());
    // }

    entity::add_child(hand, item_instance.spawn());
}
