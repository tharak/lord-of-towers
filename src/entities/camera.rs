//! Constructs camera presentation entities from Bevy primitives.
//! Child visuals follow their parent lifetime; scene infrastructure survives restart.
use crate::components::{EntityKind, SpectatorCamera};
use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub(crate) fn spawn_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Name::new("Spectator camera"),
            EntityKind::Camera,
            SpectatorCamera,
            Camera3d::default(),
            bevy::core_pipeline::tonemapping::Tonemapping::SomewhatBoringDisplayTransform,
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: 120.0,
                    min_height: 120.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_xyz(60.0, 90.0, 90.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id()
}
