use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::prelude::*;

use packages::{
    actions::messages::*,
    items::components::held_ref,
    map::components::{chunk, chunk_tile_refs},
    player::components::{left_hand_ref, right_hand_ref},
};

mod shared;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionTarget {
    MediumCrop(EntityId),
    Crafting,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActionContext {
    pub primary_held: EntityId,
    pub secondary_held: EntityId,
}

impl ActionContext {
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
pub struct ActionCallback {
    pub module: EntityId,
    pub id: String,
}

pub type Registry<T> = Arc<Mutex<T>>;
pub type ActionStore = HashMap<ActionContext, ActionCallback>;
pub type TargetStore = Registry<HashMap<ActionTarget, ActionStore>>;

#[derive(Clone, Default)]
pub struct ActionRegistry {
    pub targets: TargetStore,
}

impl ActionRegistry {
    pub fn register_action(
        &self,
        target: ActionTarget,
        context: ActionContext,
        cb: ActionCallback,
    ) {
        let mut targets = self.targets.lock().unwrap();
        let store = targets.entry(target).or_default();

        if store.contains_key(&context) {
            eprintln!("action on {:?} already registered: {:?}", target, context);
        }

        eprintln!(
            "registering {:?} action for {:?}: {:?}",
            target, context, cb
        );

        store.insert(context, cb);
    }

    pub fn perform_action(&self, target: ActionTarget, player: EntityId) {
        let targets = self.targets.lock().unwrap();
        let Some(store) = targets.get(&target) else { return };

        ActionContext::for_player_contexts(player, move |context, right_is_primary| {
            if let Some(action) = store.get(&context) {
                OnAction::new(action.id.clone(), player, right_is_primary)
                    .send_local(action.module);
                false
            } else {
                true
            }
        });
    }
}

#[main]
fn main() {
    let registry = ActionRegistry::default();

    RegisterCraftingAction::subscribe({
        let registry = registry.clone();
        move |source, data| {
            let (context, _right_is_primary) =
                ActionContext::new(data.primary_held, data.secondary_held);

            let Some(module) = source.local() else { return };
            let id = data.id;
            let cb = ActionCallback { module, id };

            registry.register_action(ActionTarget::Crafting, context, cb);
        }
    });

    RegisterMediumCropAction::subscribe({
        let registry = registry.clone();
        move |source, data| {
            let (context, _right_is_primary) =
                ActionContext::new(data.primary_held, data.secondary_held);

            let Some(module) = source.local() else { return };
            let id = data.id;
            let cb = ActionCallback { module, id };

            registry.register_action(ActionTarget::MediumCrop(data.class), context, cb);
        }
    });

    PerformCraftingAction::subscribe({
        let registry = registry.clone();
        move |source, _data| {
            let Some(player) = source.client_entity_id() else { return };
            registry.perform_action(ActionTarget::Crafting, player);
        }
    });

    let chunks = flowerpot_common::init_map(chunk());
    PerformTileAction::subscribe({
        let registry = registry.clone();
        move |source, data| {
            let Some(player) = source.client_entity_id() else { return };

            if !data.on_occupant {
                // TODO tile actions are unimplemented
                return;
            }

            let chunks = chunks.read().unwrap();

            let Some(chunk) = chunks.get(&data.chunk_pos) else {
                eprintln!("tile action on chunk {} is OOB", data.chunk_pos);
                return;
            };

            let Some(tiles) = entity::get_component(*chunk, chunk_tile_refs()) else {
                return;
            };

            let Some(tile) = tiles.get(data.tile_idx as usize) else {
                eprintln!("tile index {} is OOB", data.tile_idx);
                return;
            };

            use crate::packages::crops::components::*;
            let Some(occupant) = entity::get_component(*tile, medium_crop_occupant()) else { return };
            let Some(class) = entity::get_component(occupant, class()) else { return };

            registry.perform_action(ActionTarget::MediumCrop(class), player);
        }
    });

    PerformSwap::subscribe(move |source, _data| {
        let Some(player) = source.client_entity_id() else { return };
        let Some(left) = entity::get_component(player, left_hand_ref()) else { return };
        let Some(right) = entity::get_component(player, right_hand_ref()) else { return };

        let left_held = entity::get_component(left, held_ref()).unwrap_or_default();
        let right_held = entity::get_component(right, held_ref()).unwrap_or_default();

        entity::add_component(left, held_ref(), right_held);
        entity::add_component(right, held_ref(), left_held);
    });
}
