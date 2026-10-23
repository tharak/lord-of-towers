//! Constructs faction entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{EntityKind, Faction, MatchEntity};
use bevy::prelude::*;

pub(crate) fn spawn_faction(commands: &mut Commands, index: usize, name: &'static str) -> Entity {
    commands
        .spawn((
            Name::new(name),
            EntityKind::Faction,
            MatchEntity,
            Faction { index },
        ))
        .id()
}
