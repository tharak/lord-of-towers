//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default)]
pub enum GatheringState {
    #[default]
    Exploring,
    Following {
        trip: u64,
        sequence: u32,
    },
    Returning {
        tower: Option<Entity>,
    },
}
