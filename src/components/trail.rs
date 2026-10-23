//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Trail {
    pub trip: u64,
    pub sequence: u32,
    pub remaining: f32,
}
