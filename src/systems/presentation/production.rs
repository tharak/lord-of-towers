use crate::components::{ElementInventory, Health, PlayerControlled, ProductionIndicator, Tower};
use crate::entities::production_indicator::spawn_production_indicator;
use crate::resources::VisualAssets;
use crate::systems::presentation::stacks::tower_top;
use crate::types::{ELEMENTS_PER_MINION, Element};
use bevy::prelude::*;

pub(crate) fn sync_production_indicators(world: &mut World) {
    let towers: Vec<_> = world
        .query_filtered::<(
            Entity,
            &ElementInventory,
            Option<&PlayerControlled>,
            Option<&Health>,
        ), With<Tower>>()
        .iter(world)
        .map(|(entity, stock, player, health)| {
            (
                entity,
                player.is_some() && stock.total() >= ELEMENTS_PER_MINION,
                tower_top(health) + 1.6,
            )
        })
        .collect();
    for (parent, available, height) in towers {
        let existing = world.get::<Children>(parent).and_then(|children| {
            children
                .iter()
                .find(|child| world.get::<ProductionIndicator>(*child).is_some())
        });
        if !available {
            if let Some(child) = existing {
                world.despawn(child);
            }
        } else if let Some(child) = existing {
            world
                .entity_mut(child)
                .insert(Transform::from_xyz(0.0, height, 0.0).with_scale(Vec3::splat(2.0)));
        } else {
            let assets = world.resource::<VisualAssets>();
            let mesh = assets.sphere.clone();
            let material = assets.elements[Element::Air as usize].clone();
            spawn_production_indicator(world, parent, height, mesh, material);
        }
    }
}
