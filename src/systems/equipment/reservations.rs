use crate::components::{Allegiance, ElementInventory, EquipmentOrder, Health};
use crate::entities::element_deposit::drop_cargo;
use bevy::prelude::*;

pub(crate) fn live(world: &World, entity: Entity) -> bool {
    world.get::<Health>(entity).is_some_and(|h| h.current > 0.0)
}
pub(crate) fn drop_units(world: &mut World, position: Vec2, inventory: ElementInventory) {
    if inventory.is_empty() {
        return;
    }
    drop_cargo(&mut world.commands(), position, inventory);
}
pub(crate) fn finish_order(world: &mut World, entity: Entity, refund: bool) {
    let Some(order) = world.get::<EquipmentOrder>(entity).copied() else {
        return;
    };
    let inventory = world
        .get::<ElementInventory>(entity)
        .cloned()
        .unwrap_or_default();
    if refund && live(world, order.tower) {
        if let Some(mut stock) = world.get_mut::<ElementInventory>(order.tower) {
            stock.units.extend(inventory.units);
        }
    } else {
        drop_units(world, order.source_position, inventory);
    }
    world.despawn(entity);
}
pub(crate) fn cleanup_orders(world: &mut World) {
    let orders: Vec<_> = world
        .query::<(Entity, &EquipmentOrder)>()
        .iter(world)
        .map(|(e, o)| (e, *o))
        .collect();
    for (entity, order) in orders {
        let valid = live(world, order.minion)
            && live(world, order.tower)
            && world.get::<Allegiance>(order.minion) == world.get::<Allegiance>(order.tower);
        if !valid {
            // Destruction always drops reservations at their recorded source.
            let refund = live(world, order.minion) && live(world, order.tower);
            finish_order(world, entity, refund);
        }
    }
}
