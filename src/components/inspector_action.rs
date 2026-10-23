//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub(crate) enum InspectorAction {
    Kind,
    Faction,
    Engine,
    Previous,
    Next,
    Components,
    Close,
}
