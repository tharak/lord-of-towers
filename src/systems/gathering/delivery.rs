use crate::components::{
    Allegiance, DragControl, ElementInventory, EquipmentOrder, GatheringState, Movement, Position,
    TrailEmitter,
};
use crate::resources::SimulationConfig;
use crate::systems::gathering::navigation::units;
use bevy::prelude::*;
use std::collections::HashSet;

pub(crate) fn deliver(world: &mut World) {
    let ordered: HashSet<Entity> = world
        .query::<&EquipmentOrder>()
        .iter(world)
        .map(|o| o.minion)
        .collect();
    let radius = world.resource::<SimulationConfig>().delivery_radius;
    for entity in units(world) {
        if ordered.contains(&entity) || world.get::<DragControl>(entity).is_some() {
            continue;
        }
        let Some(GatheringState::Returning { tower: Some(tower) }) =
            world.get::<GatheringState>(entity).copied()
        else {
            continue;
        };
        let (Some(p), Some(tp), Some(a), Some(ta), Some(cargo)) = (
            world.get::<Position>(entity),
            world.get::<Position>(tower),
            world.get::<Allegiance>(entity),
            world.get::<Allegiance>(tower),
            world.get::<ElementInventory>(entity),
        ) else {
            continue;
        };
        if cargo.is_empty() || a != ta || p.0.distance_squared(tp.0) > radius * radius {
            continue;
        }
        let cargo = cargo.clone();
        if let Some(mut stock) = world.get_mut::<ElementInventory>(tower) {
            stock.units.extend(cargo.units);
        } else {
            continue;
        }
        world
            .entity_mut(entity)
            .insert((ElementInventory::default(), GatheringState::Exploring))
            .remove::<TrailEmitter>();
        if let Some(mut movement) = world.get_mut::<Movement>(entity) {
            movement.retarget_in = 0.0;
        }
    }
}
