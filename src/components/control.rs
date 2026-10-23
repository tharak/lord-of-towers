//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub(crate) enum Control {
    Pause,
    Speed(f32),
    Restart,
    Hud,
}
