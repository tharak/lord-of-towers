use crate::components::{DonutVisual, ElementInventory, Health, Minion, Stack, Tower};
use crate::entities::donut_visual::spawn_donut_visual;
use crate::resources::VisualAssets;
use crate::systems::presentation::stacks::tower_top;
use crate::types::Element;
use bevy::prelude::*;

// Minion cargo stays in eight-piece crown tiers.
pub(crate) fn donut_transform(index: usize) -> Transform {
    let angle = (index % 8) as f32 * std::f32::consts::TAU / 8.0;
    let radius = 0.65;
    let base = 1.15;
    Transform::from_xyz(
        angle.cos() * radius,
        base + (index / 8) as f32 * 0.24,
        angle.sin() * radius,
    )
    .with_scale(Vec3::splat(0.36))
}
// Exclusive reconciliation makes additions/removals visible in this same frame.
pub(crate) fn sync_donut_visuals(world: &mut World) {
    let assets = world.resource::<VisualAssets>();
    let mesh = assets.donut.clone();
    let materials = assets.elements.clone();
    let inventories: Vec<_> = world
        .query_filtered::<(Entity, &ElementInventory, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>()
        .iter(world)
        .map(|(entity, raw, tower)| (entity, raw.clone(), tower.is_some()))
        .collect();
    for (parent, inventory, tower) in inventories {
        // Group presentation by type without changing the authoritative FIFO stock.
        let counts = inventory.counts();
        let donuts: Vec<_> = if tower {
            Element::ALL
                .into_iter()
                .flat_map(|element| {
                    (0..counts[element as usize] as usize).map(move |level| (element, level))
                })
                .collect()
        } else {
            inventory
                .units
                .iter()
                .copied()
                .enumerate()
                .map(|(index, element)| (element, index))
                .collect()
        };
        let count = donuts.len();
        let mut children: Vec<Entity> = world
            .get::<Children>(parent)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.get::<DonutVisual>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default();
        let retained = count.min(children.len());
        for child in children.drain(retained..) {
            world.despawn(child);
        }
        let offset = if tower {
            tower_top(world.get::<Health>(parent)) + 0.6
        } else {
            0.0
        };
        let transform_for = |element: Element, level: usize| {
            let mut transform = if tower {
                let angle =
                    element as usize as f32 * std::f32::consts::TAU / Element::ALL.len() as f32;
                let stack = Stack {
                    count: counts[element as usize] as usize,
                    columns: 1,
                    spacing: Vec3::new(0.0, 0.24, 0.0),
                };
                let mut transform = stack.transform(level).with_scale(Vec3::splat(0.36));
                transform.translation += Vec3::new(angle.cos() * 1.3, 1.0, angle.sin() * 1.3);
                transform
            } else {
                donut_transform(level)
            };
            transform.translation.y += offset;
            transform
        };
        for (index, child) in children.iter().enumerate() {
            let (element, level) = donuts[index];
            let transform = transform_for(element, level);
            world.entity_mut(*child).insert((
                Name::new(format!(
                    "{} crown donut {}",
                    donuts[index].0.name(),
                    index + 1
                )),
                transform,
                MeshMaterial3d(materials[donuts[index].0 as usize].clone()),
            ));
        }
        for index in children.len()..count {
            spawn_donut_visual(
                world,
                parent,
                index,
                donuts[index].0,
                mesh.clone(),
                materials[donuts[index].0 as usize].clone(),
                transform_for(donuts[index].0, donuts[index].1),
            );
        }
    }
}
