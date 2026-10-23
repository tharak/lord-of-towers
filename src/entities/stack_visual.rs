//! Constructs stack visual presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{EntityKind, StackPiece};
use bevy::prelude::*;

pub(crate) fn spawn_stack_visual(
    world: &mut World,
    parent: Entity,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Name::new("Stack primitive"),
            EntityKind::StackVisual,
            StackPiece,
            ChildOf(parent),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
        ))
        .id()
}
