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
    pub fn new(mut primary_held: EntityId, mut secondary_held: EntityId) -> Self {
        if primary_held < secondary_held {
            std::mem::swap(&mut primary_held, &mut secondary_held);
        }

        Self {
            primary_held,
            secondary_held,
        }
    }

    pub fn get_player_contexts(player: EntityId) -> [Option<Self>; 3] {
        let Some(left_hand) = entity::get_component(player, left_hand_ref()) else { return [None; 3] };
        let Some(right_hand) = entity::get_component(player, right_hand_ref()) else { return [None; 3] };

        let left_held = entity::get_component(left_hand, held_ref()).unwrap_or_default();
        let right_held = entity::get_component(right_hand, held_ref()).unwrap_or_default();

        let both = Self::new(left_held, right_held);
        let right = Self::new(right_held, EntityId::null());
        let left = Self::new(left_held, EntityId::null());

        [
            Some(both.clone()),
            if right != both { Some(right) } else { None },
            if left != both { Some(left) } else { None },
        ]
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

            let context = CraftingActionContext::new(data.primary_held, data.secondary_held);

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
            for context in CraftingActionContext::get_player_contexts(player) {
                let Some(context) = context else { continue };
                let Some(action) = registry.get(&context) else { continue };
                OnCraftingAction::new(action.id.clone(), player).send_local(action.module);
                break;
            }
        }
    });
}
