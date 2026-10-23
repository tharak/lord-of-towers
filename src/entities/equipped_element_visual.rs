//! Constructs equipped element visual presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{EntityKind, EquippedElementVisual, MatchEntity};
use crate::types::Element;
use bevy::prelude::*;

pub(crate) fn spawn_equipped_element_visual(
    world: &mut World,
    parent: Entity,
    slot: usize,
    element: Element,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Name::new(format!("Equipped {} slot {}", element.name(), slot + 1)),
            EntityKind::EquippedElementVisual,
            EquippedElementVisual { slot, element },
            MatchEntity,
            ChildOf(parent),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
        ))
        .id()
}
