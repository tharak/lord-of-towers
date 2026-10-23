use crate::components::{
    Allegiance, ElementDeposit, ElementInventory, Faction, Health, Stack, StackPiece, Tower,
    TowerBeacon,
};
use crate::entities::stack_visual::spawn_stack_visual;
use crate::entities::tower_beacon::spawn_tower_beacon;
use crate::resources::VisualAssets;
use crate::types::Element;
use bevy::prelude::*;

pub(crate) fn health_count(health: Option<&Health>) -> usize {
    health.map_or(0, |health| health.current.max(0.0).floor() as usize)
}
pub(crate) fn tower_top(health: Option<&Health>) -> f32 {
    health_count(health).div_ceil(16) as f32 * 0.68
}
pub(crate) fn sync_stacks(world: &mut World) {
    let parents: Vec<_> = world
        .query_filtered::<(
            Entity,
            Option<&Tower>,
            Option<&Health>,
            Option<&ElementInventory>,
            Option<&Allegiance>,
        ), Or<(With<Tower>, With<ElementDeposit>)>>()
        .iter(world)
        .map(|(entity, tower, health, raw, allegiance)| {
            (
                entity,
                tower.is_some(),
                health_count(health),
                raw.cloned().unwrap_or_default(),
                allegiance.copied(),
            )
        })
        .collect();
    for (parent, tower, health, raw, allegiance) in parents {
        let stack = Stack {
            count: if tower { health } else { raw.total() },
            columns: if tower { 4 } else { 1 },
            spacing: if tower {
                Vec3::splat(0.68)
            } else {
                Vec3::new(0.0, 0.24, 0.0)
            },
        };
        world.entity_mut(parent).insert(stack);
        let assets = world.resource::<VisualAssets>();
        let mesh = if tower {
            assets.tower.clone()
        } else {
            assets.deposit.clone()
        };
        let material = allegiance
            .and_then(|a| world.get::<Faction>(a.0))
            .map_or_else(
                || assets.elements[Element::Fire as usize].clone(),
                |f| assets.materials[f.index].clone(),
            );
        let beacon_mesh = assets.sphere.clone();
        let element_materials = assets.elements.clone();
        let children: Vec<_> = world
            .get::<Children>(parent)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.get::<StackPiece>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default();
        for child in children.iter().skip(stack.count) {
            world.despawn(*child);
        }
        for index in 0..stack.count {
            let transform = stack.transform(index);
            let material = if tower {
                material.clone()
            } else {
                element_materials[raw.units[index] as usize].clone()
            };
            if let Some(child) = children.get(index) {
                world
                    .entity_mut(*child)
                    .insert((transform, MeshMaterial3d(material.clone())));
            } else {
                spawn_stack_visual(world, parent, mesh.clone(), material.clone(), transform);
            }
        }
        if tower {
            let beacon = world.get::<Children>(parent).and_then(|children| {
                children
                    .iter()
                    .find(|child| world.get::<TowerBeacon>(*child).is_some())
            });
            let transform = Transform::from_xyz(0.0, health.div_ceil(16) as f32 * 0.68 + 0.45, 0.0)
                .with_scale(Vec3::splat(2.0));
            if let Some(beacon) = beacon {
                world
                    .entity_mut(beacon)
                    .insert((transform, MeshMaterial3d(material)));
            } else {
                spawn_tower_beacon(world, parent, beacon_mesh, material, transform);
            }
        }
    }
}
