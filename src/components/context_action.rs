//! ECS component data; behavior lives in systems.
use crate::types::{Element, PlayerCommand};
use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub(crate) enum ContextAction {
    Equipment {
        command: PlayerCommand,
        cancel: bool,
    },
    Recipe {
        tower: Entity,
        element: Element,
    },
}
