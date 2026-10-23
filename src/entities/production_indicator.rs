//! Constructs production indicator presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{EntityKind, MatchEntity, ProductionIndicator};
use bevy::prelude::*;

pub(crate) fn spawn_production_indicator(
    world: &mut World,
    parent: Entity,
    height: f32,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) -> Entity {
    world
        .spawn((
            Name::new("Create minion available"),
            EntityKind::ProductionIndicator,
            ProductionIndicator,
            MatchEntity,
            ChildOf(parent),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, height, 0.0).with_scale(Vec3::splat(2.0)),
        ))
        .id()
}
