use ambient_api::prelude::*;

use crate::packages::{
    fauna::components::{pitch, yaw},
    map::components::position,
    this::components::*,
};

// TODO this should be a concept-as-a-struct
pub struct PlayerState {
    pub position: Vec2,
    pub speed: f32,
}

impl PlayerState {
    pub fn get(e: EntityId) -> Option<Self> {
        Some(Self {
            position: entity::get_component(e, position())?,
            speed: entity::get_component(e, speed())?,
        })
    }

    pub fn set(&self, e: EntityId) {
        entity::add_component(e, position(), self.position);
        entity::add_component(e, speed(), self.speed);
    }

    pub fn apply(&mut self, input: &InputState, dt: f32) {
        let rotate = Mat2::from_angle(input.yaw);
        let delta = rotate * input.direction * self.speed;
        self.position += delta * dt;
    }
}

// TODO this should also be a concept-as-a-struct
pub struct InputState {
    pub direction: Vec2,
    pub pitch: f32,
    pub yaw: f32,
}

impl InputState {
    pub fn get(e: EntityId) -> Option<Self> {
        Some(Self {
            direction: entity::get_component(e, direction())?,
            pitch: entity::get_component(e, pitch())?,
            yaw: entity::get_component(e, yaw())?,
        })
    }

    pub fn set(&self, e: EntityId) {
        entity::add_component(e, direction(), self.direction);
        entity::add_component(e, pitch(), self.pitch);
        entity::add_component(e, yaw(), self.yaw);
    }
}

/// Utility function to apply movement logic to a single entity.
///
/// Returns `None` if any necessary components were missing.
pub fn update_player(e: EntityId, dt: f32) -> Option<PlayerState> {
    let mut state = PlayerState::get(e)?;
    let input = InputState::get(e)?;
    state.apply(&input, dt);
    state.set(e);
    Some(state)
}
