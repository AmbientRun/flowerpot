use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::prelude::*;

use components::{
    items::held_ref,
    player::{left_hand_ref, right_hand_ref},
};
use messages::{OnCraftingAction, PerformCraftingAction, RegisterCraftingAction};

mod shared;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CraftingActionContext {
    pub primary_held: EntityId,
    pub secondary_held: EntityId,
}

impl CraftingActionContext {
    pub fn new(left_held: EntityId, right_held: EntityId) -> (Self, bool) {
        if left_held < right_held {
            (
                Self {
                    primary_held: right_held,
                    secondary_held: left_held,
                },
                true,
            )
        } else {
            (
                Self {
                    primary_held: left_held,
                    secondary_held: right_held,
                },
                false,
            )
        }
    }

    pub fn for_player_contexts(player: EntityId, mut cb: impl FnMut(Self, bool) -> bool) {
        let Some(left_hand) = entity::get_component(player, left_hand_ref()) else { return };
        let Some(right_hand) = entity::get_component(player, right_hand_ref()) else { return };

        let left_held = entity::get_component(left_hand, held_ref()).unwrap_or_default();
        let right_held = entity::get_component(right_hand, held_ref()).unwrap_or_default();

        let (both, right_is_primary) = Self::new(left_held, right_held);
        if !cb(both, right_is_primary) {
            return;
        }

        let (right, _) = Self::new(right_held, EntityId::null());
        if right != both && !cb(right, true) {
            return;
        }

        let (left, _) = Self::new(left_held, EntityId::null());
        if left != both && !cb(left, false) {
            return;
        }
    }
}

#[derive(Debug)]
pub struct Action {
    pub module: EntityId,
    pub id: String,
}

/// Wrapper type around a shared registry.
pub type Registry<T> = Arc<Mutex<T>>;

pub type CraftingActionRegistry = Registry<HashMap<CraftingActionContext, Action>>;

pub struct ActionRegistry {
    pub crafting: HashMap<CraftingActionContext, Action>,
}

#[main]
fn main() {
    let crafting = CraftingActionRegistry::default();

    RegisterCraftingAction::subscribe({
        let crafting = crafting.clone();
        move |source, data| {
            let mut registry = crafting.lock().unwrap();

            let (context, _right_is_primary) =
                CraftingActionContext::new(data.primary_held, data.secondary_held);

            if registry.contains_key(&context) {
                eprintln!("crafting action already registered: {:?}", context);
                return;
            }

            let Some(module) = source.local() else { return };
            let id = data.id;
            let action = Action { module, id };

            eprintln!("registering crafting action {:?}: {:?}", context, action);
            registry.insert(context, action);
        }
    });

    PerformCraftingAction::subscribe({
        let crafting = crafting.clone();
        move |source, _data| {
            let Some(player) = source.client_entity_id() else { return };
            let registry = crafting.lock().unwrap();
            CraftingActionContext::for_player_contexts(player, move |context, right_is_primary| {
                if let Some(action) = registry.get(&context) {
                    OnCraftingAction::new(action.id.clone(), player, right_is_primary)
                        .send_local(action.module);
                    false
                } else {
                    true
                }
            });
        }
    });
}
