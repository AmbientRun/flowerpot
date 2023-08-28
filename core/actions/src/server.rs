use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::prelude::*;

use flowerpot_common::ActorExt;
use packages::{
    items::components::held_ref,
    map::components::{chunk, chunk_tile_refs},
    player::components::{left_hand_ref, right_hand_ref},
    things::components::class_ref,
    this::messages::*,
};

mod shared;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionTarget {
    MediumCrop(EntityId),
    Tile,
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

    pub fn for_player_contexts<T>(
        player: EntityId,
        mut cb: impl FnMut(Self, bool) -> Option<T>,
    ) -> Option<T> {
        let left_hand = entity::get_component(player, left_hand_ref())?;
        let right_hand = entity::get_component(player, right_hand_ref())?;

        let left_held = entity::get_component(left_hand, held_ref()).unwrap_or_default();
        let right_held = entity::get_component(right_hand, held_ref()).unwrap_or_default();

        let (both, right_is_primary) = Self::new(left_held, right_held);
        if let Some(result) = cb(both, right_is_primary) {
            return Some(result);
        }

        let (right, _) = Self::new(right_held, EntityId::null());
        if right != both {
            if let Some(result) = cb(right, true) {
                return Some(result);
            }
        }

        let (left, _) = Self::new(left_held, EntityId::null());
        if left != both {
            if let Some(result) = cb(left, false) {
                return Some(result);
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct ActionCallback {
    pub module: EntityId,
    pub id: String,
}

pub type Registry<T> = Arc<Mutex<T>>;
pub type ActionStore = HashMap<ActionContext, ActionCallback>;

#[derive(Clone, Default)]
pub struct ActionRegistry {
    pub targets: HashMap<ActionTarget, ActionStore>,
}

impl ActionRegistry {
    pub fn register_action(
        &mut self,
        target: ActionTarget,
        context: ActionContext,
        cb: ActionCallback,
    ) {
        let store = self.targets.entry(target).or_default();

        if store.contains_key(&context) {
            eprintln!("action on {:?} already registered: {:?}", target, context);
        }

        eprintln!(
            "registering {:?} action for {:?}: {:?}",
            target, context, cb
        );

        store.insert(context, cb);
    }

    pub fn perform_action(
        &self,
        target: ActionTarget,
        player: EntityId,
    ) -> Option<(ActionCallback, bool)> {
        let store = self.targets.get(&target)?;

        ActionContext::for_player_contexts(player, move |context, right_is_primary| {
            store.get(&context).map(|cb| (cb.clone(), right_is_primary))
        })
    }
}

#[main]
fn main() {
    let registry: Arc<Mutex<ActionRegistry>> = Default::default();

    registry.on_local_message(move |registry, module, data: RegisterCraftingAction| {
        let (context, _right_is_primary) =
            ActionContext::new(data.primary_held, data.secondary_held);

        let id = data.id;
        let cb = ActionCallback { module, id };

        registry.register_action(ActionTarget::Crafting, context, cb);
    });

    registry.on_local_message(move |registry, module, data: RegisterMediumCropAction| {
        let (context, _right_is_primary) =
            ActionContext::new(data.primary_held, data.secondary_held);

        let id = data.id;
        let cb = ActionCallback { module, id };

        registry.register_action(ActionTarget::MediumCrop(data.class), context, cb);
    });

    registry.on_local_message(move |registry, module, data: RegisterTileAction| {
        let (context, _right_is_primary) =
            ActionContext::new(data.primary_held, data.secondary_held);

        let id = data.id;
        let cb = ActionCallback { module, id };

        registry.register_action(ActionTarget::Tile, context, cb);
    });

    registry.on_client_message(move |registry, player, _data: PerformCraftingAction| {
        registry.perform_action(ActionTarget::Crafting, player);
    });

    let chunks = flowerpot_common::init_map(chunk());
    registry.on_client_message(move |registry, player, data: PerformTileAction| {
        let chunks = chunks.lock().unwrap();

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

        if data.on_occupant {
            use crate::packages::crops::components::*;
            let Some(occupant) = entity::get_component(*tile, medium_crop_occupant()) else {
                return;
            };

            let Some(class) = entity::get_component(occupant, class_ref()) else {
                return;
            };

            let Some((cb, right_is_primary)) = registry.perform_action(ActionTarget::Tile, player)
            else {
                return;
            };

            OnAction::new(cb.id, player, right_is_primary, *tile).send_local(cb.module);
        }
    });

    PerformSwap::subscribe(move |source, _data| {
        let Some(player) = source.client_entity_id() else {
            return;
        };
        let Some(left) = entity::get_component(player, left_hand_ref()) else {
            return;
        };
        let Some(right) = entity::get_component(player, right_hand_ref()) else {
            return;
        };

        let left_held = entity::get_component(left, held_ref()).unwrap_or_default();
        let right_held = entity::get_component(right, held_ref()).unwrap_or_default();

        entity::add_component(left, held_ref(), right_held);
        entity::add_component(right, held_ref(), left_held);
    });
}
