use crate::components::{Allegiance, Faction, Health, Minion, Position, Tower};
use crate::resources::VisualAssets;
use crate::types::HEALTH_PERCENT_STEPS;
use bevy::prelude::*;

pub(crate) fn attach_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    entities: Query<
        (Entity, &Allegiance, Option<&Tower>),
        (Or<(With<Tower>, With<Minion>)>, Without<Transform>),
    >,
) {
    for (entity, allegiance, tower) in &entities {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                MeshMaterial3d(assets.materials[faction.index].clone()),
                Transform::default(),
                Visibility::Inherited,
            ));
            if tower.is_none() {
                commands
                    .entity(entity)
                    .insert(Mesh3d(assets.minion.clone()));
            }
        }
    }
}
pub(crate) fn sync_visuals(
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    mut entities: Query<
        (
            &Position,
            &Allegiance,
            Option<&Tower>,
            &Health,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
) {
    for (position, allegiance, tower, health, mut transform, mut material) in &mut entities {
        transform.translation = Vec3::new(
            position.0.x,
            if tower.is_some() { 0.0 } else { 0.77 },
            position.0.y,
        );
        if let Ok(faction) = factions.get(allegiance.0) {
            let handle = if tower.is_some() {
                &assets.materials[faction.index]
            } else {
                let fraction = if health.max > 0.0 {
                    (health.current / health.max).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let step = (fraction * HEALTH_PERCENT_STEPS as f32).round() as usize;
                &assets.minion_health_materials[faction.index][step]
            };
            if material.0 != *handle {
                material.0 = handle.clone();
            }
        }
    }
}
