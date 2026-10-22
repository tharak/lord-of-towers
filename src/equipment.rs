//! Physical equipment reservations, transfer orders, and derived attributes.
use crate::{
    model::*,
    simulation::{RandomStream, SpawnSequence, spawn_minion},
};
use bevy::prelude::*;

pub struct EquipmentPlugin;
impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommands>()
            .init_resource::<PlayerSelection>()
            .add_systems(
                PreUpdate,
                cleanup_orders.after(crate::simulation::restart_match),
            )
            .add_systems(
                FixedUpdate,
                (process_commands, recompute_attributes)
                    .chain()
                    .in_set(SimSet::PlayerCommands),
            )
            .add_systems(
                FixedUpdate,
                navigate_orders.in_set(SimSet::EquipmentNavigation),
            )
            .add_systems(
                FixedUpdate,
                (transfer_orders, equip_npcs)
                    .chain()
                    .in_set(SimSet::EquipmentTransfers),
            )
            .add_systems(FixedUpdate, recompute_attributes.in_set(SimSet::Attributes))
            .add_systems(
                FixedUpdate,
                cleanup_orders
                    .before(SimSet::Victory)
                    .after(SimSet::Attributes),
            );
    }
}

fn live(world: &World, entity: Entity) -> bool {
    world.get::<Health>(entity).is_some_and(|h| h.current > 0.0)
}
fn player_owner(world: &mut World) -> Option<Entity> {
    world
        .query_filtered::<(&Allegiance, &Health), (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .find(|(_, h)| h.current > 0.0)
        .map(|(a, _)| a.0)
}
fn valid_pair(world: &World, minion: Entity, tower: Entity, owner: Entity) -> bool {
    world.get::<Minion>(minion).is_some()
        && world.get::<Tower>(tower).is_some()
        && live(world, minion)
        && live(world, tower)
        && world.get::<Allegiance>(minion) == Some(&Allegiance(owner))
        && world.get::<Allegiance>(tower) == Some(&Allegiance(owner))
}
fn drop_units(world: &mut World, position: Vec2, inventory: ElementInventory) {
    if inventory.is_empty() {
        return;
    }
    crate::gathering::drop_cargo(&mut world.commands(), position, inventory);
}
fn finish_order(world: &mut World, entity: Entity, refund: bool) {
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
fn cleanup_orders(world: &mut World) {
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
fn process_commands(world: &mut World) {
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
            let trip = crate::gathering::allocate_trip(world);
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
        world.spawn((
            Name::new("Equipment order"),
            MatchEntity,
            EntityKind::EquipmentOrder,
            EquipmentOrder {
                minion,
                tower,
                source_position,
                action,
            },
            reserved,
        ));
    }
}
fn navigate_orders(world: &mut World) {
    let orders: Vec<_> = world
        .query::<&EquipmentOrder>()
        .iter(world)
        .copied()
        .collect();
    for order in orders {
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
fn transfer_orders(world: &mut World) {
    cleanup_orders(world);
    let radius = world.resource::<SimulationConfig>().delivery_radius;
    let orders: Vec<_> = world
        .query::<(Entity, &EquipmentOrder)>()
        .iter(world)
        .map(|(e, o)| (e, *o))
        .collect();
    for (entity, order) in orders {
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
fn equip_npcs(world: &mut World) {
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
fn recompute_attributes(
    config: Res<SimulationConfig>,
    mut minions: Query<
        (
            &EquippedElements,
            &mut Health,
            &mut Attack,
            &mut Movement,
            &mut Vision,
        ),
        With<Minion>,
    >,
) {
    for (gear, mut health, mut attack, mut movement, mut vision) in &mut minions {
        let fraction = if health.max > 0.0 {
            health.current / health.max
        } else {
            0.0
        };
        health.max = config.minion_health * gear.multiplier(Element::Earth);
        health.current = fraction * health.max;
        attack.damage = 10.0 * gear.multiplier(Element::Fire);
        movement.speed = config.minion_speed * gear.multiplier(Element::Water);
        vision.radius = config.vision_radius * gear.multiplier(Element::Air);
    }
}
