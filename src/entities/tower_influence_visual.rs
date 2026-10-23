//! Constructs tower influence visual presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{EntityKind, TowerInfluenceVisual};
use bevy::prelude::*;

pub(crate) fn spawn_tower_influence_visual(
    world: &mut World,
    parent: Entity,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Name::new("Tower influence area"),
            EntityKind::TowerInfluenceVisual,
            TowerInfluenceVisual,
            ChildOf(parent),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
        ))
        .id()
}
