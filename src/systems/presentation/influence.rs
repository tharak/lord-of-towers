use crate::components::{
    Allegiance, Faction, Health, InfluenceEmitter, Tower, TowerInfluenceVisual,
};
use crate::entities::tower_influence_visual::spawn_tower_influence_visual;
use crate::resources::VisualAssets;
use bevy::prelude::*;

pub(crate) fn sync_tower_influence_visuals(world: &mut World) {
    let towers: Vec<_> = world
        .query_filtered::<(Entity, &Health, &InfluenceEmitter, &Allegiance), With<Tower>>()
        .iter(world)
        .map(|(entity, health, emitter, allegiance)| {
            (entity, health.current, emitter.radius, allegiance.0)
        })
        .collect();
    for (parent, health, radius, faction) in towers {
        let child = world.get::<Children>(parent).and_then(|children| {
            children
                .iter()
                .find(|child| world.get::<TowerInfluenceVisual>(*child).is_some())
        });
        if health <= 0.0 || !radius.is_finite() || radius <= 0.0 {
            if let Some(child) = child {
                world.despawn(child);
            }
            continue;
        }
        let Some(faction) = world.get::<Faction>(faction) else {
            continue;
        };
        let assets = world.resource::<VisualAssets>();
        let material = assets.influence_materials[faction.index].clone();
        let mesh = assets.influence_ring.clone();
        let transform =
            Transform::from_xyz(0.0, 0.08, 0.0).with_scale(Vec3::new(radius, 0.1, radius));
        if let Some(child) = child {
            world
                .entity_mut(child)
                .insert((transform, MeshMaterial3d(material)));
        } else {
            spawn_tower_influence_visual(world, parent, mesh, material, transform);
        }
    }
}
