//! Decorative, primitive-only map geometry. No collision or gameplay components.
use crate::components::{EntityKind, MapDecoration, MapScene};
use crate::resources::MapVisualAssets;
use crate::types::{
    map_data::{MapData, MapPropKind},
    map_settings::MapClimate,
};
use bevy::prelude::*;
pub(crate) fn spawn_map_scene(
    commands: &mut Commands,
    data: &MapData,
    assets: &MapVisualAssets,
    climate: MapClimate,
) {
    let leaves = match climate {
        MapClimate::Temperate => 0,
        MapClimate::Arid => 1,
        MapClimate::Snow => 2,
    };
    commands
        .spawn((
            Name::new("Generated map scenery"),
            EntityKind::MapDecoration,
            MapScene,
            Transform::default(),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            for prop in &data.props {
                let position = Vec3::new(prop.position.x, 0.0, prop.position.y);
                let rotation = Quat::from_rotation_y(prop.angle);
                let mut piece = |name: &'static str,
                                 mesh: Handle<Mesh>,
                                 material: Handle<StandardMaterial>,
                                 offset: Vec3,
                                 size: Vec3| {
                    parent.spawn((
                        Name::new(name),
                        EntityKind::MapDecoration,
                        MapDecoration,
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform {
                            translation: position + offset,
                            rotation,
                            scale: size,
                        },
                    ));
                };
                match prop.kind {
                    MapPropKind::Building => {
                        piece(
                            "Settlement building",
                            assets.cube.clone(),
                            assets.walls.clone(),
                            Vec3::Y * (prop.size.y * 0.5),
                            prop.size,
                        );
                        piece(
                            "Settlement roof",
                            assets.canopy.clone(),
                            assets.roof.clone(),
                            Vec3::Y * (prop.size.y + 0.12),
                            Vec3::new(prop.size.x * 0.8, 0.3, prop.size.z * 0.8),
                        );
                    }
                    MapPropKind::Tree => {
                        piece(
                            "Tree trunk",
                            assets.cube.clone(),
                            assets.trunk.clone(),
                            Vec3::Y * (prop.size.y * 0.3),
                            Vec3::new(0.13, prop.size.y * 0.6, 0.13),
                        );
                        piece(
                            "Tree canopy",
                            assets.canopy.clone(),
                            assets.leaves[leaves].clone(),
                            Vec3::Y * (prop.size.y * 0.7),
                            prop.size,
                        );
                    }
                    MapPropKind::Rock => piece(
                        "Landscape rock",
                        assets.rock.clone(),
                        assets.stone.clone(),
                        Vec3::Y * 0.15,
                        Vec3::new(prop.size.x, 0.4, prop.size.z),
                    ),
                }
            }
        });
}
