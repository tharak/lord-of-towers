use crate::components::{DragControl, EquipmentOrder, Movement, Position};
use crate::systems::equipment::reservations::live;
use bevy::prelude::*;

pub(crate) fn navigate_orders(world: &mut World) {
    let orders: Vec<_> = world
        .query::<&EquipmentOrder>()
        .iter(world)
        .copied()
        .collect();
    for order in orders {
        if world.get::<DragControl>(order.minion).is_some() {
            continue;
        }
        if !live(world, order.tower) {
            continue;
        }
        let Some(position) = world.get::<Position>(order.tower).copied() else {
            continue;
        };
        if let Some(mut movement) = world.get_mut::<Movement>(order.minion) {
            movement.target = position.0;
            movement.retarget_in = 0.0;
        }
    }
}
