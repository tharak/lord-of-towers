use crate::components::{Allegiance, Faction, Position, Projectile};
use crate::resources::VisualAssets;
use bevy::prelude::*;

pub(crate) fn attach_projectile_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    projectiles: Query<(Entity, &Allegiance), (With<Projectile>, Without<Mesh3d>)>,
) {
    for (entity, allegiance) in &projectiles {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                Mesh3d(assets.sphere.clone()),
                MeshMaterial3d(assets.materials[faction.index].clone()),
                Transform::default(),
            ));
        }
    }
}
pub(crate) fn sync_projectile_visuals(
    mut projectiles: Query<(&Position, &mut Transform), With<Projectile>>,
) {
    for (position, mut transform) in &mut projectiles {
        transform.translation = Vec3::new(position.0.x, 1.0, position.0.y);
    }
}
