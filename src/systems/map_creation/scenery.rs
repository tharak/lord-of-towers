use crate::components::MapScene;
use crate::entities::map_scene::spawn_map_scene;
use crate::resources::{MapCreation, MapLandscape, MapVisualAssets};
use bevy::prelude::*;
pub(crate) fn setup_map_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 1.0,
            ..default()
        })
    };
    commands.insert_resource(MapVisualAssets {
        cube: meshes.add(Cuboid::default()),
        canopy: meshes.add(Cone::new(1.0, 1.0).mesh().resolution(4)),
        rock: meshes.add(Sphere::new(0.5).mesh().uv(8, 5)),
        walls: material(Color::srgb(0.68, 0.61, 0.46)),
        roof: material(Color::srgb(0.37, 0.17, 0.12)),
        trunk: material(Color::srgb(0.26, 0.18, 0.11)),
        leaves: [
            material(Color::srgb(0.16, 0.34, 0.17)),
            material(Color::srgb(0.40, 0.40, 0.17)),
            material(Color::srgb(0.69, 0.78, 0.75)),
        ],
        stone: material(Color::srgb(0.40, 0.43, 0.42)),
    });
}
pub(crate) fn update_map_scene(
    mut commands: Commands,
    map: Res<MapCreation>,
    landscape: Res<MapLandscape>,
    assets: Res<MapVisualAssets>,
    scenes: Query<Entity, With<MapScene>>,
    mut revision: Local<u64>,
) {
    if *revision == landscape.revision {
        return;
    }
    for entity in &scenes {
        commands.entity(entity).despawn();
    }
    if let Some(data) = &landscape.data {
        spawn_map_scene(&mut commands, data, &assets, map.settings.climate);
    }
    *revision = landscape.revision;
}
