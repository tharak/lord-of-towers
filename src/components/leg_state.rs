//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Animation-facing locomotion state, independent of simultaneous arm actions.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegState {
    #[default]
    Idle,
    Moving,
    DragStart,
    DragRelease,
}
