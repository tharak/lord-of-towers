//! Constructs tower beacon presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{EntityKind, TowerBeacon};
use bevy::prelude::*;

pub(crate) fn spawn_tower_beacon(
    world: &mut World,
    parent: Entity,
    beacon_mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Name::new("Tower beacon"),
            EntityKind::TowerBeacon,
            TowerBeacon,
            ChildOf(parent),
            Mesh3d(beacon_mesh),
            MeshMaterial3d(material),
            transform,
        ))
        .id()
}
