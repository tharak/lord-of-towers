//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Movement {
    pub speed: f32,
    pub target: Vec2,
    pub retarget_in: f32,
}
