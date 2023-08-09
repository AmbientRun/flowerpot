use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ambient_api::{components::core::player::user_id, ecs::SupportedValue, prelude::*};

mod shared;

use components::{
    fauna::*,
    map::{in_chunk, players_observing, position},
};
use messages::*;

#[main]
fn main() {
    let _store = ChunkOccupants::new();

    bind_fauna_update(position(), move |e, player, position| {
        UpdateFaunaPosition::new(e, position).send_client_targeted_reliable(player);
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
        .requires(fauna())
        .bind({
            let cb = cb.clone();
            move |entities| {
                for (e, (chunk, data)) in entities {
                    for player in
                        entity::get_component(chunk, players_observing()).unwrap_or_default()
                    {
                        let Some(uid) = entity::get_component(player, user_id()) else { continue };
                        cb(e, uid, data.clone());
                    }
                }
            }
        });

    OnSpawnFauna::subscribe(move |source, spawn| {
        if source.local().is_none() {
            return;
        }

        if let Some(data) = entity::get_component(spawn.fauna, component) {
            cb(spawn.fauna, spawn.player_uid, data);
        }
    });
}

pub struct ChunkOccupants {
    chunks_to_occupants: HashMap<EntityId, HashSet<EntityId>>,
    occupants_to_chunks: HashMap<EntityId, EntityId>,
}

impl ChunkOccupants {
    pub fn new() -> Arc<Mutex<Self>> {
        let store = Self {
            chunks_to_occupants: Default::default(),
            occupants_to_chunks: Default::default(),
        };

        let store = Arc::new(Mutex::new(store));

        spawn_query(in_chunk()).requires(fauna()).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.lock().unwrap();
                for (e, chunk) in entities {
                    store.update_occupant(e, chunk);
                }
            }
        });

        change_query(in_chunk())
            .track_change(in_chunk())
            .requires(fauna())
            .bind({
                let store = store.clone();
                move |entities| {
                    let mut store = store.lock().unwrap();
                    for (e, new_chunk) in entities {
                        store.update_occupant(e, new_chunk);
                    }
                }
            });

        despawn_query(()).requires((fauna(), in_chunk())).bind({
            let store = store.clone();
            move |entities| {
                let mut store = store.lock().unwrap();
                for (e, _) in entities {
                    store.remove_occupant(e);
                }
            }
        });

        OnPlayerLoadChunk::subscribe({
            let store = store.clone();
            move |_, data| {
                let store = store.lock().unwrap();
                if let Some(occupants) = store.chunks_to_occupants.get(&data.chunk_entity) {
                    for occupant in occupants.iter() {
                        Self::spawn_to_observer(*occupant, data.player_entity);
                    }
                }
            }
        });

        OnPlayerUnloadChunk::subscribe({
            let store = store.clone();
            move |_, data| {
                let store = store.lock().unwrap();
                if let Some(occupants) = store.chunks_to_occupants.get(&data.chunk_entity) {
                    for occupant in occupants.iter() {
                        Self::despawn_from_observer(*occupant, data.player_entity);
                    }
                }
            }
        });

        store
    }

    fn update_occupant(&mut self, occupant: EntityId, new_chunk: EntityId) {
        if let Some(old_chunk) = self.occupants_to_chunks.insert(occupant, new_chunk) {
            if old_chunk != new_chunk {
                self.chunks_to_occupants
                    .get_mut(&old_chunk)
                    .unwrap()
                    .remove(&occupant);

                self.on_move(occupant, old_chunk, new_chunk);
            }
        } else {
            for observer in Self::get_observers(new_chunk) {
                Self::spawn_to_observer(occupant, observer);
            }
        }

        self.chunks_to_occupants
            .entry(new_chunk)
            .or_default()
            .insert(occupant);
    }

    fn remove_occupant(&mut self, occupant: EntityId) {
        if let Some(chunk) = self.occupants_to_chunks.remove(&occupant) {
            self.chunks_to_occupants
                .get_mut(&chunk)
                .unwrap_or_else(|| {
                    panic!(
                        "attempted to remove occupant {} from non-existent chunk {}",
                        occupant, chunk
                    )
                })
                .remove(&occupant);

            for observer in Self::get_observers(chunk) {
                Self::despawn_from_observer(occupant, observer);
            }
        }
    }

    fn on_move(&self, occupant: EntityId, old_chunk: EntityId, new_chunk: EntityId) {
        let old_observers = Self::get_observers(old_chunk).into_iter();
        let new_observers = Self::get_observers(new_chunk).into_iter();
        let on_old = |old: &EntityId| Self::despawn_from_observer(occupant, *old);
        let on_new = |new: &EntityId| Self::spawn_to_observer(occupant, *new);
        flowerpot::diff_sorted(old_observers, new_observers, on_old, on_new);
    }

    fn get_observers(chunk: EntityId) -> Vec<EntityId> {
        entity::get_component(chunk, players_observing()).unwrap_or_default()
    }

    fn despawn_from_observer(e: EntityId, player_entity: EntityId) {
        let Some(player_uid) = entity::get_component(player_entity, user_id()) else { return };
        DespawnFauna::new(e).send_client_targeted_reliable(player_uid);
    }

    fn spawn_to_observer(e: EntityId, player_entity: EntityId) {
        let Some(player_uid) = entity::get_component(player_entity, user_id()) else { return };
        SpawnFauna::new(e).send_client_targeted_reliable(player_uid.clone());
        OnSpawnFauna::new(e, player_entity, player_uid.clone()).send_local_broadcast(true);
    }
}
