use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::prelude::*;

use packages::{
    actions::messages::*,
    items::components::held_ref,
    player::components::{left_hand_ref, right_hand_ref},
    this::components::*,
};

mod shared;

pub struct CraftingYield {
    pub primary: EntityId,
    pub secondary: EntityId,
    pub write_secondary: bool,
}

#[main]
fn main() {
    let store = Arc::new(Mutex::new(HashMap::<String, CraftingYield>::new()));

    spawn_query((
        is_recipe(),
        primary_ingredient(),
        secondary_ingredient(),
        primary_yield(),
        secondary_yield(),
    ))
    .bind({
        let store = store.clone();
        move |entities| {
            let mut store = store.lock().unwrap();
            for (
                e,
                (_recipe, primary_ingredient, secondary_ingredient, primary_yield, secondary_yield),
            ) in entities
            {
                let id = e.to_string();
                let crafting_yield = CraftingYield {
                    primary: primary_yield,
                    secondary: secondary_yield,
                    write_secondary: secondary_yield != secondary_ingredient,
                };

                store.insert(id.clone(), crafting_yield);

                RegisterCraftingAction::new(id, primary_ingredient, secondary_ingredient)
                    .send_local_broadcast(false);
            }
        }
    });

    OnAction::subscribe({
        let store = store.clone();
        move |_, data| {
            if let Some(crafting_yield) = store.lock().unwrap().get(&data.id) {
                let (primary, secondary) = if data.right_is_primary {
                    (right_hand_ref(), left_hand_ref())
                } else {
                    (left_hand_ref(), right_hand_ref())
                };

                let Some(primary) = entity::get_component(data.player, primary) else {
                    return;
                };
                let Some(secondary) = entity::get_component(data.player, secondary) else {
                    return;
                };

                entity::add_component(primary, held_ref(), crafting_yield.primary);

                if crafting_yield.write_secondary {
                    entity::add_component(secondary, held_ref(), crafting_yield.secondary);
                }
            }
        }
    });
}
