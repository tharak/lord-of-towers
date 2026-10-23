use crate::{
    components::MapVisionRoot, entities::map_vision::spawn_map_vision, resources::MapLandscape,
};
use bevy::prelude::*;

/// Sight geometry has its own lifetime; destroying scenery cannot change simulation vision.
pub(crate) fn install_map_vision(
    mut commands: Commands,
    landscape: Res<MapLandscape>,
    roots: Query<Entity, With<MapVisionRoot>>,
    mut revision: Local<u64>,
) {
    if *revision == landscape.revision {
        return;
    }
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    if let Some(data) = &landscape.data {
        spawn_map_vision(&mut commands, data);
    }
    *revision = landscape.revision;
}
