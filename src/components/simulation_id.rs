//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Stable across match restart; also resolves deterministic combat and collection ties.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimulationId(pub u64);
