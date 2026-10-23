//! Constructs light presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::EntityKind;
use bevy::prelude::*;

pub(crate) fn spawn_light(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Name::new("Sun"),
            EntityKind::Light,
            DirectionalLight {
                illuminance: 12000.0,
                ..default()
            },
            Transform::from_xyz(20.0, 50.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id()
}
