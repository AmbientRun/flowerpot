use std::{
    collections::VecDeque,
    f32::consts::FRAC_PI_2,
    sync::{Arc, Mutex},
};

use ambient_api::{
    core::{
        app::components::main_scene,
        camera::components::aspect_ratio_from_window,
        camera::concepts::make_perspective_infinite_reverse_camera,
        messages::Frame,
        player::components::{is_player, local_user_id, user_id},
        transform::{
            components::{local_to_parent, local_to_world, rotation, scale, translation},
            concepts::make_transformable,
        },
    },
    prelude::*,
};

use flowerpot_common::{ActorExt, CHUNK_SIZE};
use packages::{
    fauna::components::{pitch, yaw},
    map::components::{chunk, chunk_tile_index, in_chunk, position},
    terrain::{
        components::{altitude, highlight_tile},
        messages::{RaycastRequest, RaycastResponse},
    },
    this::{components::*, messages::*},
};

use shared::*;

use crate::packages::map::components::chunk_tile_refs;

mod shared;

// TODO make a component?
const HEAD_HEIGHT: f32 = 1.5;

#[main]
fn main() {
    spawn_query((is_player(), user_id())).bind(move |players| {
        let local_user_id = entity::get_component(entity::resources(), local_user_id()).unwrap();
        for (player_entity, (_, user)) in players {
            if user != local_user_id {
                continue;
            }

            let head = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with(main_scene(), ())
                .with(user_id(), user.clone())
                .with(translation(), Vec3::Z * HEAD_HEIGHT)
                .with(local_to_parent(), Mat4::IDENTITY)
                .with(rotation(), Quat::from_rotation_x(FRAC_PI_2))
                .spawn();

            entity::add_child(player_entity, head);

            let init_hand = |hand_ref, offset| {
                let hand = Entity::new()
                    .with(main_scene(), ())
                    .with(local_to_parent(), Mat4::IDENTITY)
                    .with(local_to_world(), Mat4::IDENTITY)
                    .with(translation(), offset)
                    .with(rotation(), Quat::from_rotation_x(-FRAC_PI_2))
                    .with(scale(), Vec3::splat(0.3))
                    .spawn();

                entity::add_child(head, hand);

                entity::add_component(player_entity, hand_ref, hand);
            };

            init_hand(left_hand_ref(), Vec3::new(-0.5, -0.4, 1.0));
            init_hand(right_hand_ref(), Vec3::new(0.5, -0.4, 1.0));

            entity::add_components(
                player_entity,
                Entity::new()
                    .with_merge(make_transformable())
                    .with(local_to_world(), Mat4::IDENTITY)
                    .with(position(), Vec2::ZERO)
                    .with(speed(), 10.0) // TODO terrain-based speed
                    .with(head_ref(), head),
            );

            entity::add_component(entity::resources(), local_player_ref(), player_entity);
        }
    });

    change_query((is_player(), yaw(), pitch()))
        .track_change((yaw(), pitch()))
        .bind(move |players| {
            for (e, (_player, yaw, pitch)) in players {
                entity::add_component(e, rotation(), Quat::from_rotation_z(yaw));
                if let Some(head) = entity::get_component(e, head_ref()) {
                    entity::add_component(
                        head,
                        rotation(),
                        Quat::from_rotation_x(FRAC_PI_2 + pitch),
                    );
                }
            }
        });

    change_query((is_player(), position(), altitude()))
        .track_change((position(), altitude()))
        .bind(move |entities| {
            for (e, (_, position, altitude)) in entities {
                let new_translation = position.extend(altitude);
                entity::add_component(e, translation(), new_translation);
            }
        });

    let input = Arc::new(Mutex::new(InputPrediction::new()));

    Frame::subscribe({
        let input = input.clone();
        move |_| {
            input.lock().unwrap().on_frame();
        }
    });

    input.on_message(move |input, _, data: UpdatePlayerState| {
        let state = PlayerState {
            position: data.position,
            speed: data.speed,
        };

        input.rewind(data.sequence, state);
    });

    query((yaw(), pitch(), position(), altitude()))
        .requires(is_player())
        .each_frame(move |players| {
            for (_e, (yaw, pitch, position, altitude)) in players {
                let origin = position.extend(altitude + 1.5); // TODO player height component
                let delta = Quat::from_rotation_z(yaw) * Quat::from_rotation_x(pitch) * -Vec3::Y;
                let limit = 10.0; // TODO player reach

                RaycastRequest::new(origin, delta, limit).send_local_broadcast(false);
            }
        });

    let mut last_tile: Option<(EntityId, u8)> = None;
    RaycastResponse::subscribe(move |_, data| {
        if let Some((last_chunk, last_tile_idx)) = last_tile {
            if last_chunk != data.chunk_entity || last_tile_idx != data.tile_idx {
                if let Some(last_highlight) =
                    entity::get_component(player::get_local(), tile_selection_ref())
                {
                    entity::despawn_recursive(last_highlight);
                }

                entity::remove_component(player::get_local(), tile_selection_ref());
                last_tile = None;
            } else {
                // last tile is the currently-highlighted one, so skip
                // highlighting a new one
                return;
            }
        }

        if data.distance < 0.0 {
            return;
        }

        let highlight = Entity::new()
            .with(in_chunk(), data.chunk_entity)
            .with(chunk_tile_index(), data.tile_idx)
            .with(highlight_tile(), ())
            .with(
                ambient_api::core::app::components::name(),
                "Highlighted Tile".into(),
            )
            .spawn();

        last_tile = Some((data.chunk_entity, data.tile_idx));
        entity::add_component(player::get_local(), tile_selection_ref(), highlight);
    });

    let chunks = flowerpot_common::init_map(chunk());
    let mut sequence = 0;
    chunks.on_message(move |chunks, _, data: UpdateLoadedChunks| {
        if data.sequence < sequence {
            eprintln!("received out-of-sequence chunk update: {}", data.sequence);
        }

        sequence = data.sequence;

        let old_chunks =
            entity::get_component(player::get_local(), loaded_chunks()).unwrap_or_default();

        // TODO use sorted diffs

        for new in data.chunks.iter() {
            if !old_chunks.contains(new) {
                let mut tiles = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
                for _y in 0..CHUNK_SIZE {
                    for _x in 0..CHUNK_SIZE {
                        tiles.push(Entity::new().spawn());
                    }
                }

                Entity::new()
                    .with(chunk(), *new)
                    .with(chunk_tile_refs(), tiles)
                    .spawn();
            }
        }

        for old in old_chunks.iter() {
            if !data.chunks.contains(old) {
                if let Some(chunk) = chunks.get(old) {
                    entity::despawn_recursive(*chunk);
                }
            }
        }

        entity::add_component(player::get_local(), loaded_chunks(), data.chunks);
    });
}

pub struct InputStep {
    pub state: InputState,
    pub dt: f32,
}

pub struct InputPrediction {
    local_sequence: u64,
    inputs: VecDeque<InputStep>,
    last_server_update: u64,
    state: PlayerState,
    e: EntityId,
}

impl InputPrediction {
    pub fn new() -> Self {
        Self {
            local_sequence: 0,
            inputs: VecDeque::new(),
            last_server_update: 0,
            state: PlayerState {
                position: Vec2::new(0.0, 0.0),
                speed: 1.0,
            },
            e: player::get_local(),
        }
    }

    pub fn on_frame(&mut self) {
        let dt = delta_time();

        let state = match InputState::get(self.e) {
            Some(state) => state,
            None => {
                // eprintln!("local player has incomplete input state");
                return;
            }
        };

        self.apply(InputStep { state, dt });
    }

    pub fn apply(&mut self, input: InputStep) {
        UpdatePlayerInput {
            direction: input.state.direction,
            pitch: input.state.pitch,
            yaw: input.state.yaw,
            sequence: self.local_sequence,
        }
        .send_server_unreliable();

        self.state.apply(&input.state, input.dt);
        self.state.set(self.e);
        self.inputs.push_front(input);
        self.local_sequence += 1;
    }

    pub fn rewind(&mut self, sequence: u64, mut state: PlayerState) {
        // if this update is out of order, drop it
        if self.last_server_update > sequence {
            return;
        }

        // drop obsolete updates
        let skipped = sequence - self.last_server_update;
        for _ in 0..skipped {
            self.inputs.pop_back();
        }

        // update the sequence of the latest server update
        self.last_server_update = sequence;

        // rewind
        for input in self.inputs.iter().rev() {
            state.apply(&input.state, input.dt);
        }

        // apply new state
        self.update_state(state);
    }

    pub fn update_state(&mut self, state: PlayerState) {
        state.set(self.e);
        self.state = state;
    }
}
