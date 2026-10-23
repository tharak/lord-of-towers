//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Maximum local sight radius; map footprints can occlude targets within it.
#[derive(Component, Clone, Copy, Debug)]
pub struct Vision {
    pub radius: f32,
}
