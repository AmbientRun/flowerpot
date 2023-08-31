use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ambient_api::{
    core::{network::components::no_sync, player::components::user_id},
    prelude::*,
};
use flowerpot_common::{ActorExt, SystemExt};

use packages::this::{components::*, messages::*};

#[derive(Default)]
pub struct RegionOccupants {
    regions_to_occupants: HashMap<EntityId, HashSet<EntityId>>,
    occupants_to_regions: HashMap<EntityId, EntityId>,
}

impl RegionOccupants {
    fn update_occupant(&mut self, occupant: EntityId, new_region: EntityId) {
        if let Some(old_region) = self.occupants_to_regions.insert(occupant, new_region) {
            if old_region != new_region {
                self.regions_to_occupants
                    .get_mut(&old_region)
                    .unwrap()
                    .remove(&occupant);

                self.on_move(occupant, old_region, new_region);
            }
        } else {
            for observer in Self::get_observers(new_region) {
                Self::spawn_to_observer(occupant, observer);
            }
        }

        self.regions_to_occupants
            .entry(new_region)
            .or_default()
            .insert(occupant);
    }

    fn remove_region(&mut self, region: EntityId) {
        let Some(occupants) = self.regions_to_occupants.remove(&region) else {
            return;
        };

        let observers = Self::get_observers(region);

        for occupant in occupants {
            let removed = self.occupants_to_regions.remove(&occupant);
            if removed.is_some() {
                for observer in observers.iter() {
                    Self::despawn_from_observer(occupant, *observer);
                }
            }
        }
    }

    fn remove_occupant(&mut self, occupant: EntityId) {
        if let Some(region) = self.occupants_to_regions.remove(&occupant) {
            self.regions_to_occupants
                .get_mut(&region)
                .unwrap_or_else(|| {
                    panic!(
                        "attempted to remove occupant {} from non-existent region {}",
                        occupant, region
                    )
                })
                .remove(&occupant);

            for observer in Self::get_observers(region) {
                Self::despawn_from_observer(occupant, observer);
            }
        }
    }

    fn on_move(&self, occupant: EntityId, old_region: EntityId, new_region: EntityId) {
        let old_observers = Self::get_observers(old_region).into_iter();
        let new_observers = Self::get_observers(new_region).into_iter();
        let on_old = |old: &EntityId| Self::despawn_from_observer(occupant, *old);
        let on_new = |new: &EntityId| Self::spawn_to_observer(occupant, *new);
        flowerpot_common::diff_sorted(old_observers, new_observers, on_old, on_new);
    }

    fn get_observers(region: EntityId) -> Vec<EntityId> {
        entity::get_component(region, players_observing()).unwrap_or_default()
    }

    fn despawn_from_observer(e: EntityId, player_entity: EntityId) {
        if e == player_entity {
            // do not sync players to themselves
            return;
        }

        let Some(player_uid) = entity::get_component(player_entity, user_id()) else {
            return;
        };

        DespawnThing::new(e).send_client_targeted_reliable(player_uid);
    }

    fn spawn_to_observer(e: EntityId, player_entity: EntityId) {
        if e == player_entity {
            // do not sync players to themselves
            return;
        }

        let Some(player_uid) = entity::get_component(player_entity, user_id()) else {
            return;
        };

        SpawnThing::new(e).send_client_targeted_reliable(player_uid.clone());
        OnSpawnThing::new(e, player_entity, player_uid.clone()).send_local_broadcast(true);
    }
}

#[main]
fn main() {
    let occupants: Arc<Mutex<RegionOccupants>> = Default::default();

    occupants.on_event(spawn_query(in_region()), |occupants, e, region| {
        entity::add_component(e, no_sync(), ());
        occupants.update_occupant(e, region);
    });

    occupants.on_change(
        change_query(in_region()).track_change(in_region()),
        RegionOccupants::update_occupant,
    );

    occupants.on_event(
        despawn_query(()).requires(in_region()),
        move |occupants, e, _| occupants.remove_occupant(e),
    );

    occupants.on_event(
        despawn_query(()).requires(players_observing()),
        move |occupants, e, _| occupants.remove_region(e),
    );

    occupants.on_local_message(move |occupants, _, data: LoadPlayerRegion| {
        entity::mutate_component_with_default(
            data.region,
            players_observing(),
            Vec::new(),
            move |observing| {
                let mut added = false;
                for (idx, p) in observing.iter().enumerate() {
                    if *p > data.player_entity {
                        observing.insert(idx, data.player_entity);
                        added = true;
                        break;
                    } else if *p == data.player_entity {
                        return;
                    }
                }

                if !added {
                    observing.push(data.player_entity);
                }
            },
        );

        if let Some(occupants) = occupants.regions_to_occupants.get(&data.region) {
            for occupant in occupants.iter() {
                RegionOccupants::spawn_to_observer(*occupant, data.player_entity);
            }
        }
    });

    occupants.on_local_message(move |occupants, _, data: UnloadPlayerRegion| {
        entity::mutate_component_with_default(
            data.region,
            players_observing(),
            Vec::new(),
            move |observing| {
                observing.retain(|p| *p != data.player_entity);
            },
        );

        if let Some(occupants) = occupants.regions_to_occupants.get(&data.region) {
            for occupant in occupants.iter() {
                RegionOccupants::despawn_from_observer(*occupant, data.player_entity);
            }
        }
    });

    // debug system to assert that region observer lists are always valid
    change_query(players_observing())
        .track_change(players_observing())
        .bind(move |entities| {
            for (e, observing) in entities {
                let is_sorted = observing.windows(2).all(|w| w[0] < w[1]);
                if !is_sorted {
                    eprintln!(
                        "list of observers to region {} isn't sorted: {:?}",
                        e, observing
                    );
                }
            }
        });
}
