//! ECS component data; behavior lives in systems.
use crate::types::Element;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct EquippedElementVisual {
    pub(crate) slot: usize,
    pub(crate) element: Element,
}
