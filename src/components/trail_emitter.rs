//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct TrailEmitter {
    pub trip: u64,
    pub next_sequence: u32,
    pub last_position: Vec2,
}
