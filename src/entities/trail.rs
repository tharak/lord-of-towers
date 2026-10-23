//! Constructs trail entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{Allegiance, EntityKind, MatchEntity, Position, Trail};
use crate::resources::SimulationConfig;
use bevy::prelude::*;

pub(crate) fn marker(
    world: &mut World,
    position: Vec2,
    owner: Allegiance,
    trip: u64,
    sequence: u32,
) -> Entity {
    let remaining = world.resource::<SimulationConfig>().trail_lifetime;
    world
        .spawn((
            Name::new("Element trail"),
            MatchEntity,
            EntityKind::Trail,
            Position(position),
            owner,
            Trail {
                trip,
                sequence,
                remaining,
            },
        ))
        .id()
}
