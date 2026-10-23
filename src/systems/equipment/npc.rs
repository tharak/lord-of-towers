use crate::components::{
    Allegiance, DragControl, ElementInventory, EquippedElements, Minion, PlayerControlled,
    Position, SimulationId, Tower,
};
use crate::resources::SimulationConfig;
use crate::systems::equipment::reservations::live;
use bevy::prelude::*;

pub(crate) fn equip_npcs(world: &mut World) {
    let radius = world.resource::<SimulationConfig>().delivery_radius;
    let mut towers: Vec<_> = world
        .query_filtered::<(Entity, &SimulationId), (With<Tower>, Without<PlayerControlled>)>()
        .iter(world)
        .map(|(e, id)| (id.0, e))
        .collect();
    towers.sort_by_key(|v| v.0);
    let mut minions: Vec<_> = world
        .query_filtered::<(Entity, &SimulationId), With<Minion>>()
        .iter(world)
        .map(|(e, id)| (id.0, e))
        .collect();
    minions.sort_by_key(|v| v.0);
    for (_, tower) in towers {
        if !live(world, tower) {
            continue;
        }
        let owner = world.get::<Allegiance>(tower).copied();
        let position = world.get::<Position>(tower).unwrap().0;
        for &(_, minion) in &minions {
            if !live(world, minion)
                || world.get::<DragControl>(minion).is_some()
                || world.get::<Allegiance>(minion).copied() != owner
                || world
                    .get::<Position>(minion)
                    .unwrap()
                    .0
                    .distance_squared(position)
                    > radius * radius
            {
                continue;
            }
            for slot in 0..3 {
                if world
                    .get::<EquippedElements>(minion)
                    .is_none_or(|g| g.slots[slot].is_some())
                {
                    continue;
                }
                let element = world
                    .get_mut::<ElementInventory>(tower)
                    .and_then(|mut stock| stock.units.pop_front());
                if let Some(element) = element {
                    world.get_mut::<EquippedElements>(minion).unwrap().slots[slot] = Some(element);
                }
            }
        }
    }
}
