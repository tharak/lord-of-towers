//! Constructs projectile entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{Allegiance, EntityKind, MatchEntity, Position, Projectile};
use bevy::prelude::*;

pub(crate) fn spawn_projectile(
    commands: &mut Commands,
    position: Vec2,
    allegiance: Allegiance,
    projectile: Projectile,
) -> Entity {
    commands
        .spawn((
            Name::new("Projectile"),
            EntityKind::Projectile,
            MatchEntity,
            Position(position),
            allegiance,
            projectile,
        ))
        .id()
}
