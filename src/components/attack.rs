//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Attack {
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
    pub remaining: f32,
}
