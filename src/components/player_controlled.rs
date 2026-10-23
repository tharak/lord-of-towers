//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// The one tower whose faction accepts player equipment and production commands.
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerControlled;
