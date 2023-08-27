use ambient_api::prelude::*;

use packages::this::components::*;

mod shared;

#[main]
fn main() {
    entity::add_component(
        entity::synchronized_resources(),
        real_time_to_game_time(),
        1000.0,
    );
    entity::add_component(entity::synchronized_resources(), time_of_day(), 12.0);

    run_async(async move {
        loop {
            let tick = 5.0f64;
            sleep(tick as f32).await;

            let to_game_time =
                entity::get_component(entity::synchronized_resources(), real_time_to_game_time())
                    .expect("real_time_to_game_time resource was removed");

            let hours = tick / 60.0 / 60.0;
            let elapsed = hours * to_game_time;

            entity::mutate_component_with_default(
                entity::synchronized_resources(),
                time_of_day(),
                12.0,
                |time| {
                    *time += elapsed;
                    while *time > 24.0 {
                        *time -= 24.0;
                    }
                },
            );
        }
    });
}
