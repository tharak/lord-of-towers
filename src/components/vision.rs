//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Vision {
    pub radius: f32,
}
