//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Each icon is built from UI rectangles so it does not depend on font glyph coverage.
#[derive(Component)]
pub(crate) struct PauseIcon(pub(crate) bool);
