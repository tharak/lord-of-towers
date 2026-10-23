//! Simulation-only sight blockers created from the same map data as scenery.
use crate::components::{EntityKind, MapVisionRoot, Position, VisionBlocker};
use crate::types::{
    VisionBlockerShape,
    map_data::{MapData, MapPropKind},
};
use bevy::prelude::*;

pub(crate) fn spawn_map_vision(commands: &mut Commands, data: &MapData) {
    commands
        .spawn((
            Name::new("Map sight blockers"),
            EntityKind::VisionObstacle,
            MapVisionRoot,
        ))
        .with_children(|parent| {
            for prop in &data.props {
                let (name, half_extents, shape) = match prop.kind {
                    MapPropKind::Building => (
                        "Building sight blocker",
                        Vec2::new(prop.size.x, prop.size.z) * 0.5,
                        VisionBlockerShape::Rectangle,
                    ),
                    MapPropKind::Tree => (
                        "Tree canopy sight blocker",
                        Vec2::new(prop.size.x, prop.size.z),
                        VisionBlockerShape::Ellipse,
                    ),
                    MapPropKind::Rock => continue,
                };
                parent.spawn((
                    Name::new(name),
                    EntityKind::VisionObstacle,
                    Position(prop.position),
                    VisionBlocker {
                        half_extents,
                        angle: prop.angle,
                        shape,
                    },
                ));
            }
        });
}
