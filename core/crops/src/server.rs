use ambient_api::prelude::*;

use packages::{
    map::components::*,
    region_networking::messages::OnSpawnThing,
    things::components::class_ref,
    this::{components::*, messages::*},
};

mod shared;

#[main]
fn main() {
    shared::init_shared();

    // despawn old crops since we don't spawn prefabs
    spawn_query(despawn_when_loaded()).bind(move |entities| {
        for (_e, old) in entities {
            entity::despawn_recursive(old);
        }
    });

    OnSpawnThing::subscribe(move |source, spawn| {
        if source.local().is_none() {
            return;
        }

        if let Some(coords) = entity::get_component(spawn.thing, coords()) {
            UpdateCropCoords::new(spawn.thing, coords)
                .send_client_targeted_reliable(spawn.player_uid);
        }
    });

    run_async(async move {
        let all_crops = query((is_medium_crop(), on_tile(), age())).build();
        loop {
            sleep(0.1).await;

            for (e, (_medium, _tile, old_age)) in all_crops.evaluate() {
                let new_age = old_age + 1;
                entity::set_component(e, age(), new_age);
            }
        }
    });

    change_query((
        is_medium_crop(),
        on_tile(),
        age(),
        seeding_interval(),
        seed(),
    ))
    .track_change(age())
    .bind(move |entities| {
        for (_e, (_, tile, age, interval, seed)) in entities {
            if age == 0 || age % interval != 0 {
                continue;
            }

            let mut neighbors = [
                north_neighbor(),
                east_neighbor(),
                south_neighbor(),
                west_neighbor(),
            ];

            let mut rng = thread_rng();
            neighbors.shuffle(&mut rng);

            for neighbor in neighbors {
                let Some(neighbor) = entity::get_component(tile, neighbor) else {
                    continue;
                };

                if !entity::get_component(neighbor, medium_crop_occupant())
                    .unwrap_or_default()
                    .is_null()
                {
                    continue;
                }

                Entity::new()
                    .with(is_medium_crop(), ())
                    .with(class_ref(), seed)
                    .with(on_tile(), neighbor)
                    .spawn();

                break;
            }
        }
    });

    change_query((
        is_medium_crop(),
        on_tile(),
        age(),
        next_growth_age(),
        next_growth_stage(),
    ))
    .track_change(age())
    .bind(move |entities| {
        for (e, (_, tile, current_age, next_age, next)) in entities {
            if current_age < next_age {
                continue;
            }

            entity::despawn_recursive(e);

            if !next.is_null() {
                Entity::new()
                    .with(is_medium_crop(), ())
                    .with(class_ref(), next)
                    .with(on_tile(), tile)
                    .spawn();
            }
        }
    });
}
