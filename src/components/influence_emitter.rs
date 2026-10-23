//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct InfluenceEmitter {
    pub radius: f32,
    pub strength: f32,
}
