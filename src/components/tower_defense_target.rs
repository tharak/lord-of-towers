//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Persistent response to a visible attacker that fired at a friendly tower.
#[derive(Component, Clone, Copy, Debug)]
pub struct TowerDefenseTarget {
    pub tower: Entity,
    pub attacker: Entity,
}
