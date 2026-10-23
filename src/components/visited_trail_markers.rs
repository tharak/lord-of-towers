//! ECS component data; behavior lives in systems.
use bevy::prelude::*;
use std::collections::HashSet;

/// Reached and bypassed marker identities prevent loops without imposing a trip direction.
#[derive(Component, Default)]
pub(crate) struct VisitedTrailMarkers(pub(crate) HashSet<Entity>);
