//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// A homing projectile. It follows a live hostile target and deals damage on impact.
#[derive(Component, Clone, Copy, Debug)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub remaining: f32,
}
