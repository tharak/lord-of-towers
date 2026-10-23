use bevy::prelude::*;

/// Owns map sight blockers independently of presentation; survives match restart.
#[derive(Component)]
pub(crate) struct MapVisionRoot;
