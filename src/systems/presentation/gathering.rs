use crate::components::{Allegiance, DonutVisual, ElementDeposit, Faction, Position, Trail};
use crate::resources::{SimulationConfig, VisualAssets};
use crate::types::TRAIL_BUCKETS;
use bevy::prelude::*;

pub(crate) fn attach_resource_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    deposits: Query<Entity, (With<ElementDeposit>, Without<Transform>)>,
    trails: Query<(Entity, &Allegiance), (With<Trail>, Without<Mesh3d>)>,
    factions: Query<&Faction>,
) {
    for entity in &deposits {
        commands
            .entity(entity)
            .insert((Transform::default(), Visibility::Inherited));
    }
    for (entity, allegiance) in &trails {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                Mesh3d(assets.trail.clone()),
                MeshMaterial3d(assets.trail_materials[faction.index][TRAIL_BUCKETS - 1].clone()),
                Transform::default(),
            ));
        }
    }
}
pub(crate) fn trail_bucket(remaining: f32, lifetime: f32) -> usize {
    ((remaining / lifetime.max(f32::EPSILON)).clamp(0.0, 1.0) * TRAIL_BUCKETS as f32)
        .ceil()
        .max(1.0) as usize
        - 1
}
pub(crate) fn sync_gathering_visuals(
    assets: Res<VisualAssets>,
    config: Res<SimulationConfig>,
    factions: Query<&Faction>,
    mut deposits: Query<(&Position, &mut Transform), (With<ElementDeposit>, Without<Trail>)>,
    mut trails: Query<
        (
            &Position,
            &Trail,
            &Allegiance,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<ElementDeposit>, Without<DonutVisual>),
    >,
) {
    for (position, mut transform) in &mut deposits {
        transform.translation = Vec3::new(position.0.x, 0.35, position.0.y);
        transform.scale = Vec3::ONE;
    }
    for (position, trail, allegiance, mut transform, mut material) in &mut trails {
        transform.translation = Vec3::new(position.0.x, 0.1, position.0.y);
        if let Ok(faction) = factions.get(allegiance.0) {
            material.0 = assets.trail_materials[faction.index]
                [trail_bucket(trail.remaining, config.trail_lifetime)]
            .clone();
        }
    }
}
