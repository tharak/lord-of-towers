//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Independent flags permit carrying and shooting at the same time.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmState {
    pub carrying: bool,
    pub shooting: bool,
}
