//! Constructs battlefield presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::EntityKind;
use bevy::prelude::*;

pub(crate) fn spawn_battlefield(
    commands: &mut Commands,
    ground: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    commands
        .spawn((
            Name::new("Battlefield"),
            EntityKind::Battlefield,
            Mesh3d(ground.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            })),
        ))
        .id()
}
