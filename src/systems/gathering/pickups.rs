use crate::components::{
    Allegiance, DragControl, ElementDeposit, ElementInventory, EquipmentOrder, EquippedElements,
    GatheringState, Minion, Position, TrailEmitter, Vision,
};
use crate::resources::SimulationConfig;
use crate::systems::gathering::navigation::{nearest_tower, stable};
use crate::systems::gathering::trails::{allocate_trip, carrier_marker};
use crate::types::TowerOrderAction;
use bevy::prelude::*;
use std::collections::HashSet;

pub(crate) fn collect(world: &mut World) {
    let radius = world.resource::<SimulationConfig>().pickup_radius;
    let reserved_slots: HashSet<_> = world
        .query::<&EquipmentOrder>()
        .iter(world)
        .filter_map(|order| match order.action {
            TowerOrderAction::Equip { slot } => Some((order.minion, slot)),
            _ => None,
        })
        .collect();
    let mut deposits: Vec<_> = world
        .query_filtered::<Entity, With<ElementDeposit>>()
        .iter(world)
        .collect();
    deposits.sort_by_key(|e| stable(world, *e));
    for deposit in deposits {
        let Some(position) = world.get::<Position>(deposit).copied() else {
            continue;
        };
        let mut candidates: Vec<_> = world
            .query_filtered::<(Entity, &Position, &ElementInventory, &Vision), (With<Minion>, Without<DragControl>)>()
            .iter(world)
            .filter_map(|(e, p, a, v)| {
                let d = p.0.distance_squared(position.0);
                (a.is_empty() && d <= radius * radius && d <= v.radius * v.radius).then_some((e, d))
            })
            .collect();
        candidates.sort_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
        });
        for (entity, _) in candidates {
            let Some(element) = world
                .get::<ElementInventory>(deposit)
                .and_then(|inventory| inventory.units.front().copied())
            else {
                break;
            };
            let Some(owner) = world.get::<Allegiance>(entity).copied() else {
                continue;
            };
            let empty_slot = world.get::<EquippedElements>(entity).and_then(|gear| {
                gear.slots.iter().enumerate().find_map(|(slot, equipped)| {
                    (equipped.is_none() && !reserved_slots.contains(&(entity, slot)))
                        .then_some(slot)
                })
            });
            if let Some(slot) = empty_slot {
                world.get_mut::<EquippedElements>(entity).unwrap().slots[slot] = Some(element);
            } else {
                let p = world.get::<Position>(entity).unwrap().0;
                let tower = nearest_tower(world, p, owner.0);
                let trip = allocate_trip(world);
                world.entity_mut(entity).insert((
                    ElementInventory::homogeneous(element, 1),
                    GatheringState::Returning { tower },
                    TrailEmitter {
                        trip,
                        next_sequence: 1,
                        last_position: p,
                    },
                ));
                carrier_marker(world, p, owner, trip, 0);
            }
            let mut stock = world.get_mut::<ElementInventory>(deposit).unwrap();
            stock.units.pop_front();
            if stock.is_empty() {
                world.despawn(deposit);
                break;
            }
        }
    }
}
