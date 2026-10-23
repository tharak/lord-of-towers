use crate::components::{
    Allegiance, ElementInventory, ElementStats, EquipmentOrder, EquippedElements, GatheringState,
    Health, Minion, PlayerControlled, Position, Tower, TrailEmitter,
};
use crate::entities::equipment_order::spawn_equipment_order;
use crate::entities::minion::spawn_minion;
use crate::resources::{
    ElementEconomy, PlayerCommands, RandomStream, SimulationConfig, SpawnSequence,
};
use crate::systems::equipment::reservations::{finish_order, live};
use crate::systems::gathering::trails::allocate_trip;
use crate::types::{ELEMENTS_PER_MINION, PlayerCommand, TowerOrderAction};
use bevy::prelude::*;

pub(crate) fn player_owner(world: &mut World) -> Option<Entity> {
    world
        .query_filtered::<(&Allegiance, &Health), (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .find(|(_, h)| h.current > 0.0)
        .map(|(a, _)| a.0)
}
pub(crate) fn valid_pair(world: &World, minion: Entity, tower: Entity, owner: Entity) -> bool {
    world.get::<Minion>(minion).is_some()
        && world.get::<Tower>(tower).is_some()
        && live(world, minion)
        && live(world, tower)
        && world.get::<Allegiance>(minion) == Some(&Allegiance(owner))
        && world.get::<Allegiance>(tower) == Some(&Allegiance(owner))
}
pub(crate) fn process_commands(world: &mut World) {
    let pending = std::mem::take(&mut world.resource_mut::<PlayerCommands>().0);
    let Some(owner) = player_owner(world) else {
        return;
    };
    for command in pending {
        if let PlayerCommand::CreateMinion { tower, counts } = command {
            if counts.iter().map(|&n| n as u64).sum::<u64>() != ELEMENTS_PER_MINION as u64
                || world.get::<Tower>(tower).is_none()
                || !live(world, tower)
                || world.get::<Allegiance>(tower) != Some(&Allegiance(owner))
            {
                continue;
            }
            let Some(stock) = world.get::<ElementInventory>(tower) else {
                continue;
            };
            if stock
                .counts()
                .iter()
                .zip(counts)
                .any(|(have, need)| *have < need)
            {
                continue;
            }
            let mut remaining = counts;
            let mut recipe = Vec::new();
            world
                .get_mut::<ElementInventory>(tower)
                .unwrap()
                .units
                .retain(|element| {
                    if remaining[*element as usize] > 0 {
                        remaining[*element as usize] -= 1;
                        recipe.push(*element);
                        false
                    } else {
                        true
                    }
                });
            for element in &recipe {
                world.resource_mut::<ElementEconomy>().consumed[*element as usize] += 1;
            }
            let config = world.resource::<SimulationConfig>().clone();
            let position = world.get::<Position>(tower).unwrap().0;
            let mut random = world.remove_resource::<RandomStream>().unwrap();
            let mut sequence = world.remove_resource::<SpawnSequence>().unwrap();
            spawn_minion(
                &mut world.commands(),
                &config,
                &mut random,
                &mut sequence,
                tower,
                position,
                Allegiance(owner),
                ElementStats::from_recipe(recipe),
            );
            world.insert_resource(random);
            world.insert_resource(sequence);
            continue;
        }
        let (minion, tower, action, requested) = match command {
            PlayerCommand::CancelOrder { minion } => {
                if world.get::<Allegiance>(minion) != Some(&Allegiance(owner)) {
                    continue;
                }
                let found = world
                    .query::<(Entity, &EquipmentOrder)>()
                    .iter(world)
                    .find(|(_, o)| o.minion == minion)
                    .map(|(e, _)| e);
                if let Some(entity) = found {
                    finish_order(world, entity, true);
                }
                continue;
            }
            PlayerCommand::Equip {
                minion,
                tower,
                slot,
                element,
            } => (
                minion,
                tower,
                TowerOrderAction::Equip { slot },
                Some(element),
            ),
            PlayerCommand::ReturnEquipment {
                minion,
                tower,
                slot,
            } => (
                minion,
                tower,
                TowerOrderAction::ReturnEquipment { slot },
                None,
            ),
            PlayerCommand::DeliverCargo { minion, tower } => {
                (minion, tower, TowerOrderAction::DeliverCargo, None)
            }
            PlayerCommand::CreateMinion { .. } => unreachable!(),
        };
        if !valid_pair(world, minion, tower, owner)
            || world
                .query::<&EquipmentOrder>()
                .iter(world)
                .any(|o| o.minion == minion)
        {
            continue;
        }
        match action {
            TowerOrderAction::Equip { slot } | TowerOrderAction::ReturnEquipment { slot }
                if slot >= 3 =>
            {
                continue;
            }
            TowerOrderAction::ReturnEquipment { slot }
                if world
                    .get::<EquippedElements>(minion)
                    .is_none_or(|g| g.slots[slot].is_none()) =>
            {
                continue;
            }
            TowerOrderAction::DeliverCargo
                if world
                    .get::<ElementInventory>(minion)
                    .is_none_or(ElementInventory::is_empty) =>
            {
                continue;
            }
            _ => (),
        }
        let mut reserved = ElementInventory::default();
        if let Some(element) = requested {
            let Some(mut stock) = world.get_mut::<ElementInventory>(tower) else {
                continue;
            };
            let Some(index) = stock.units.iter().position(|e| *e == element) else {
                continue;
            };
            reserved.units.push_back(stock.units.remove(index).unwrap());
        }
        if let TowerOrderAction::ReturnEquipment { slot } = action {
            // Unequipped units are physical cargo during the homeward trip.
            let element = world.get_mut::<EquippedElements>(minion).unwrap().slots[slot]
                .take()
                .unwrap();
            world
                .get_mut::<ElementInventory>(minion)
                .unwrap()
                .units
                .push_back(element);
            let position = world.get::<Position>(minion).unwrap().0;
            let trip = allocate_trip(world);
            world.entity_mut(minion).insert((
                GatheringState::Returning { tower: Some(tower) },
                TrailEmitter {
                    trip,
                    next_sequence: 0,
                    last_position: position,
                },
            ));
        }
        let source_position = world.get::<Position>(tower).unwrap().0;
        spawn_equipment_order(world, minion, tower, source_position, action, reserved);
    }
}
