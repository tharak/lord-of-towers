//! Constructs donut visual presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{DonutVisual, EntityKind};
use crate::types::Element;
use bevy::prelude::*;

pub(crate) fn spawn_donut_visual(
    world: &mut World,
    parent: Entity,
    index: usize,
    element: Element,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Name::new(format!("{} crown donut {}", element.name(), index + 1)),
            EntityKind::DonutVisual,
            DonutVisual,
            ChildOf(parent),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            transform,
        ))
        .id()
}
