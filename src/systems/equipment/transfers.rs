use crate::components::{
    DragControl, ElementInventory, EquipmentOrder, EquippedElements, GatheringState, Movement,
    Position, TrailEmitter,
};
use crate::resources::SimulationConfig;
use crate::systems::equipment::reservations::cleanup_orders;
use crate::types::TowerOrderAction;
use bevy::prelude::*;

pub(crate) fn transfer_orders(world: &mut World) {
    cleanup_orders(world);
    let radius = world.resource::<SimulationConfig>().delivery_radius;
    let orders: Vec<_> = world
        .query::<(Entity, &EquipmentOrder)>()
        .iter(world)
        .map(|(e, o)| (e, *o))
        .collect();
    for (entity, order) in orders {
        if world.get::<DragControl>(order.minion).is_some() {
            continue;
        }
        let (Some(minion), Some(tower)) = (
            world.get::<Position>(order.minion),
            world.get::<Position>(order.tower),
        ) else {
            continue;
        };
        if minion.0.distance_squared(tower.0) > radius * radius {
            continue;
        }
        let mut returned = Vec::new();
        match order.action {
            TowerOrderAction::Equip { slot } => {
                let element = world
                    .get_mut::<ElementInventory>(entity)
                    .and_then(|mut i| i.units.pop_front());
                if let Some(mut gear) = world.get_mut::<EquippedElements>(order.minion) {
                    if let Some(old) = std::mem::replace(&mut gear.slots[slot], element) {
                        returned.push(old);
                    }
                } else if let Some(element) = element {
                    returned.push(element);
                }
            }
            // ReturnEquipment already moved its unit into cargo when accepted.
            TowerOrderAction::ReturnEquipment { .. } | TowerOrderAction::DeliverCargo => {}
        }
        // All homeward orders deliver cargo at their actual destination.
        if let Some(mut cargo) = world.get_mut::<ElementInventory>(order.minion) {
            returned.extend(cargo.units.drain(..));
        }
        world
            .entity_mut(order.minion)
            .insert(GatheringState::Exploring)
            .remove::<TrailEmitter>();
        if let Some(mut movement) = world.get_mut::<Movement>(order.minion) {
            movement.retarget_in = 0.0;
        }
        if let Some(mut stock) = world.get_mut::<ElementInventory>(order.tower) {
            stock.units.extend(returned);
        }
        world.despawn(entity);
    }
}
