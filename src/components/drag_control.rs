//! ECS component data; behavior lives in systems.
use crate::types::DragPhase;
use bevy::prelude::*;

/// Active player movement intent; removed on arrival or input cancellation.
#[derive(Component, Clone, Copy, Debug)]
pub struct DragControl {
    pub target: Vec2,
    pub phase: DragPhase,
}
