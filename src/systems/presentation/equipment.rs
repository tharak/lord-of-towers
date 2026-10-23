use crate::components::{EquippedElementVisual, EquippedElements, Minion};
use crate::entities::equipped_element_visual::spawn_equipped_element_visual;
use crate::resources::VisualAssets;
use bevy::prelude::*;

// Equipment is independent of the carried cargo crown.
pub(crate) fn equipped_transform(slot: usize) -> Transform {
    let angle = slot as f32 * std::f32::consts::TAU / 3.0;
    Transform::from_xyz(angle.cos() * 0.95, 0.35, angle.sin() * 0.95)
}
pub(crate) fn sync_equipped_visuals(world: &mut World) {
    let inventories: Vec<_> = world
        .query_filtered::<(Entity, &EquippedElements), With<Minion>>()
        .iter(world)
        .map(|(entity, gear)| (entity, gear.slots))
        .collect();
    for (parent, slots) in inventories {
        let children: Vec<_> = world
            .get::<Children>(parent)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.get::<EquippedElementVisual>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default();
        // Three bounded children are rebuilt only when the slot layout changes.
        let expected: Vec<_> = slots
            .iter()
            .enumerate()
            .filter_map(|(slot, element)| element.map(|element| (slot, element)))
            .collect();
        let unchanged = children.len() == expected.len()
            && children
                .iter()
                .zip(&expected)
                .all(|(child, (slot, element))| {
                    world
                        .get::<EquippedElementVisual>(*child)
                        .is_some_and(|visual| visual.slot == *slot && visual.element == *element)
                });
        if unchanged {
            continue;
        }
        for child in children {
            world.despawn(child);
        }
        for (slot, element) in expected {
            let assets = world.resource::<VisualAssets>();
            let mesh = assets.sphere.clone();
            let material = assets.elements[element as usize].clone();
            spawn_equipped_element_visual(
                world,
                parent,
                slot,
                element,
                mesh,
                material,
                equipped_transform(slot),
            );
        }
    }
}
