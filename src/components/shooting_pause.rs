//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Movement pause after firing; counted in simulation seconds.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ShootingPause(pub f32);
