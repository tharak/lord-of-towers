use crate::model::*;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct NodeSnapshot {
    entity: Entity,
    id: u64,
    position: Vec2,
    tower: bool,
}
#[derive(Clone, Copy)]
struct Candidate {
    distance: f32,
    parent: usize,
    root: Entity,
}

fn precedes(
    a: Candidate,
    a_target: usize,
    b: Candidate,
    b_target: usize,
    nodes: &[NodeSnapshot],
) -> bool {
    a.distance
        .total_cmp(&b.distance)
        .then_with(|| nodes[a.parent].id.cmp(&nodes[b.parent].id))
        .then_with(|| nodes[a_target].id.cmp(&nodes[b_target].id))
        .is_lt()
}

/// Multi-root Prim expansion. Each newly connected node updates every remaining
/// candidate once, and each selection scans candidates once: O(n²), O(n) memory.
fn forest(nodes: &[NodeSnapshot], range: f32) -> Vec<(Entity, SupplyConnection)> {
    let mut connected: Vec<bool> = nodes.iter().map(|node| node.tower).collect();
    let mut candidates: Vec<Option<Candidate>> = vec![None; nodes.len()];
    let mut edges = Vec::new();
    let update =
        |source: usize, root: Entity, connected: &[bool], candidates: &mut [Option<Candidate>]| {
            for target in 0..nodes.len() {
                if connected[target] {
                    continue;
                }
                let distance = nodes[source]
                    .position
                    .distance_squared(nodes[target].position);
                if !distance.is_finite() || distance > range * range {
                    continue;
                }
                let candidate = Candidate {
                    distance,
                    parent: source,
                    root,
                };
                if candidates[target]
                    .is_none_or(|old| precedes(candidate, target, old, target, nodes))
                {
                    candidates[target] = Some(candidate);
                }
            }
        };
    for (source, node) in nodes.iter().enumerate().filter(|(_, node)| node.tower) {
        update(source, node.entity, &connected, &mut candidates);
    }
    loop {
        let mut best: Option<(usize, Candidate)> = None;
        for (target, candidate) in candidates.iter().enumerate() {
            if connected[target] {
                continue;
            }
            if let Some(candidate) = candidate
                && best.is_none_or(|(old_target, old)| {
                    precedes(*candidate, target, old, old_target, nodes)
                })
            {
                best = Some((target, *candidate));
            }
        }
        let Some((target, candidate)) = best else {
            break;
        };
        connected[target] = true;
        edges.push((
            nodes[target].entity,
            SupplyConnection {
                parent: nodes[candidate.parent].entity,
                root: candidate.root,
            },
        ));
        update(target, candidate.root, &connected, &mut candidates);
    }
    edges
}

pub(super) fn reconcile_links(world: &mut World) {
    let range = world.resource::<SimulationConfig>().link_range;
    let mut groups: BTreeMap<Entity, Vec<NodeSnapshot>> = BTreeMap::new();
    let mut query = world.query::<(
        Entity,
        &SimulationId,
        &Position,
        &Allegiance,
        Option<&Tower>,
        Option<&Minion>,
        Option<&Health>,
    )>();
    for (entity, id, position, allegiance, tower, minion, health) in query.iter(world) {
        if (tower.is_none() && minion.is_none())
            || health.is_some_and(|h| h.current <= 0.0)
            || world.get::<Faction>(allegiance.0).is_none()
        {
            continue;
        }
        groups.entry(allegiance.0).or_default().push(NodeSnapshot {
            entity,
            id: id.0,
            position: position.0,
            tower: tower.is_some(),
        });
    }
    let mut desired = BTreeMap::new();
    let mut connections = BTreeMap::new();
    if range.is_finite() && range >= 0.0 {
        for (faction, nodes) in &mut groups {
            nodes.sort_by_key(|node| node.id);
            for (target, connection) in forest(nodes, range) {
                desired.insert((connection.parent, target, *faction), ());
                connections.insert(target, connection);
            }
        }
    }
    let old_connections: Vec<_> = world
        .query_filtered::<Entity, With<SupplyConnection>>()
        .iter(world)
        .collect();
    for entity in old_connections {
        if !connections.contains_key(&entity) {
            world.entity_mut(entity).remove::<SupplyConnection>();
        }
    }
    for (entity, connection) in connections {
        if world.get::<SupplyConnection>(entity) != Some(&connection) {
            world.entity_mut(entity).insert(connection);
        }
    }
    let old_links: Vec<_> = world
        .query::<(Entity, &Link, Option<&Allegiance>)>()
        .iter(world)
        .map(|(entity, link, allegiance)| (entity, *link, allegiance.copied()))
        .collect();
    let mut retained = BTreeSet::new();
    for (entity, link, allegiance) in old_links {
        let key = allegiance.map(|faction| (link.source, link.target, faction.0));
        if key.is_some_and(|key| desired.contains_key(&key) && retained.insert(key)) {
            continue;
        }
        world.despawn(entity);
    }
    for ((source, target, faction), ()) in desired {
        if retained.contains(&(source, target, faction)) {
            continue;
        }
        world.spawn((
            Name::new(format!("Supply {source:?} -> {target:?}")),
            EntityKind::Link,
            MatchEntity,
            Allegiance(faction),
            Link { source, target },
        ));
    }
}

#[derive(Clone, Copy)]
struct SenderSnapshot {
    id: u64,
    faction: Entity,
    tower: bool,
    energy: Option<EnergyStore>,
    cursor: Option<u64>,
    connected: bool,
}

pub(super) fn transfer_energy(world: &mut World) {
    let interval = {
        let config = world.resource::<SimulationConfig>();
        config.energy_tick_seconds * config.energy_transfer_ticks as f32
    };
    if !interval.is_finite() || interval <= 0.0 {
        return;
    }
    let delta = simulation_delta(
        world.resource::<Time<Fixed>>(),
        world.resource::<MatchState>(),
    );
    world.resource_mut::<EnergyClock>().elapsed += delta;
    while world.resource::<EnergyClock>().elapsed + f32::EPSILON >= interval {
        {
            let mut clock = world.resource_mut::<EnergyClock>();
            clock.elapsed = (clock.elapsed - interval).max(0.0);
            clock.pulses += 1;
        }
        transfer_pulse(world);
    }
}

fn transfer_pulse(world: &mut World) {
    let mut nodes = BTreeMap::new();
    let mut query = world.query::<(
        Entity,
        &SimulationId,
        &Allegiance,
        Option<&Tower>,
        Option<&Minion>,
        Option<&EnergyStore>,
        Option<&ForwardingCursor>,
        Option<&SupplyConnection>,
        Option<&Health>,
    )>();
    for (entity, id, faction, tower, minion, energy, cursor, connection, health) in
        query.iter(world)
    {
        if (tower.is_none() && minion.is_none()) || health.is_some_and(|h| h.current <= 0.0) {
            continue;
        }
        let connected = connection.is_some_and(|connection| {
            world.get::<Tower>(connection.root).is_some()
                && world.get::<Allegiance>(connection.root) == Some(faction)
        });
        nodes.insert(
            entity,
            SenderSnapshot {
                id: id.0,
                faction: faction.0,
                tower: tower.is_some(),
                energy: energy.copied(),
                cursor: cursor.and_then(|c| c.last_recipient),
                connected,
            },
        );
    }
    let mut children: BTreeMap<Entity, Vec<Entity>> = BTreeMap::new();
    for link in world.query::<&Link>().iter(world) {
        let (Some(source), Some(target)) = (nodes.get(&link.source), nodes.get(&link.target))
        else {
            continue;
        };
        if source.faction != target.faction
            || target.tower
            || !target.connected
            || world
                .get::<SupplyConnection>(link.target)
                .is_none_or(|c| c.parent != link.source)
        {
            continue;
        }
        children.entry(link.source).or_default().push(link.target);
    }
    let mut changes: BTreeMap<Entity, i16> = BTreeMap::new();
    let mut cursors = Vec::new();
    for (sender, mut recipients) in children {
        let source = nodes[&sender];
        if !source.tower
            && (!source.connected || source.energy.is_none_or(|store| store.current == 0))
        {
            continue;
        }
        recipients.sort_by_key(|entity| nodes[entity].id);
        recipients.dedup();
        let eligible = |entity: &&Entity| {
            nodes[*entity]
                .energy
                .is_some_and(|store| store.current < store.capacity)
        };
        let target = recipients
            .iter()
            .filter(eligible)
            .find(|entity| source.cursor.is_none_or(|last| nodes[*entity].id > last))
            .or_else(|| recipients.iter().find(eligible));
        let Some(&target) = target else {
            continue;
        };
        if !source.tower {
            *changes.entry(sender).or_default() -= 1;
        }
        *changes.entry(target).or_default() += 1;
        cursors.push((sender, nodes[&target].id));
    }
    // Apply after choosing all senders: new energy never travels a second hop.
    for (entity, change) in changes {
        if let Some(mut store) = world.get_mut::<EnergyStore>(entity) {
            store.current = (i16::from(store.current) + change) as u8;
        }
    }
    for (entity, recipient) in cursors {
        world.entity_mut(entity).insert(ForwardingCursor {
            last_recipient: Some(recipient),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationConfig::default());
        world.insert_resource(MatchState::default());
        world.insert_resource(EnergyClock::default());
        world.insert_resource(Time::<Fixed>::from_seconds(1.0));
        world
    }
    fn faction(world: &mut World, index: usize) -> Entity {
        world.spawn(Faction { index }).id()
    }
    fn tower(world: &mut World, faction: Entity, id: u64, x: f32) -> Entity {
        world
            .spawn((
                Tower { slot: id as usize },
                SimulationId(id),
                Position(Vec2::new(x, 0.0)),
                Allegiance(faction),
                ForwardingCursor::default(),
            ))
            .id()
    }
    fn minion(world: &mut World, faction: Entity, id: u64, x: f32, current: u8) -> Entity {
        world
            .spawn((
                Minion,
                SimulationId(id),
                Position(Vec2::new(x, 0.0)),
                Allegiance(faction),
                ForwardingCursor::default(),
                EnergyStore {
                    current,
                    capacity: 5,
                },
            ))
            .id()
    }
    fn links(world: &mut World) -> Vec<(Entity, Link)> {
        world
            .query::<(Entity, &Link)>()
            .iter(world)
            .map(|(entity, link)| (entity, *link))
            .collect()
    }
    fn energy(world: &World, entity: Entity) -> u8 {
        world.get::<EnergyStore>(entity).unwrap().current
    }

    #[test]
    fn forest_is_deterministic_multi_root_and_has_no_cycles() {
        let mut world = world();
        let faction = faction(&mut world, 0);
        let left = tower(&mut world, faction, 0, -10.0);
        let right = tower(&mut world, faction, 1, 10.0);
        let mid = minion(&mut world, faction, 5, 0.0, 0);
        let far = minion(&mut world, faction, 6, 24.0, 0);
        let isolated = minion(&mut world, faction, 7, 45.0, 0);
        reconcile_links(&mut world);
        assert_eq!(world.get::<SupplyConnection>(mid).unwrap().parent, left);
        assert_eq!(world.get::<SupplyConnection>(far).unwrap().parent, right);
        assert!(world.get::<SupplyConnection>(isolated).is_none());
        assert!(world.get::<SupplyConnection>(left).is_none());
        let before = links(&mut world);
        reconcile_links(&mut world);
        assert_eq!(
            links(&mut world),
            before,
            "unchanged edges preserve entity identities"
        );
        for entity in [mid, far] {
            let mut current = entity;
            let mut visited = BTreeSet::new();
            while let Some(connection) = world.get::<SupplyConnection>(current) {
                assert!(visited.insert(current));
                current = connection.parent;
            }
            assert!(world.get::<Tower>(current).is_some());
        }
    }
    #[test]
    fn movement_death_and_capture_remove_stale_edges() {
        let mut world = world();
        let own = faction(&mut world, 0);
        let enemy = faction(&mut world, 1);
        let root = tower(&mut world, own, 0, 0.0);
        let a = minion(&mut world, own, 5, 10.0, 0);
        let b = minion(&mut world, own, 6, 20.0, 0);
        reconcile_links(&mut world);
        assert_eq!(links(&mut world).len(), 2);
        world.get_mut::<Position>(a).unwrap().0.x = 50.0;
        reconcile_links(&mut world);
        assert!(links(&mut world).is_empty());
        assert!(world.get::<SupplyConnection>(b).is_none());
        world.get_mut::<Position>(a).unwrap().0.x = 10.0;
        reconcile_links(&mut world);
        world.entity_mut(root).insert(Allegiance(enemy));
        reconcile_links(&mut world);
        assert!(links(&mut world).is_empty());
        world.entity_mut(root).insert(Allegiance(own));
        reconcile_links(&mut world);
        world.despawn(a);
        reconcile_links(&mut world);
        assert!(links(&mut world).is_empty());
        assert!(world.get::<SupplyConnection>(b).is_none());
    }
    #[test]
    fn boundary_range_and_faction_isolation() {
        let mut world = world();
        let own = faction(&mut world, 0);
        let enemy = faction(&mut world, 1);
        tower(&mut world, own, 0, 0.0);
        let border = minion(&mut world, own, 5, 15.0, 0);
        let hostile = minion(&mut world, enemy, 6, 1.0, 0);
        reconcile_links(&mut world);
        assert!(world.get::<SupplyConnection>(border).is_some());
        assert!(world.get::<SupplyConnection>(hostile).is_none());
        assert_eq!(links(&mut world).len(), 1);
    }
    #[test]
    fn tower_sends_one_total_and_rotates_skipping_full_children() {
        let mut world = world();
        let own = faction(&mut world, 0);
        let root = tower(&mut world, own, 0, 0.0);
        let a = minion(&mut world, own, 5, -10.0, 0);
        let b = minion(&mut world, own, 6, 10.0, 0);
        reconcile_links(&mut world);
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (1, 0));
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (1, 1));
        world.get_mut::<EnergyStore>(a).unwrap().current = 5;
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (5, 2));
        assert_eq!(
            world.get::<ForwardingCursor>(root).unwrap().last_recipient,
            Some(6)
        );
        world.get_mut::<EnergyStore>(b).unwrap().current = 5;
        transfer_pulse(&mut world);
        assert_eq!(
            world.get::<ForwardingCursor>(root).unwrap().last_recipient,
            Some(6)
        );
    }
    #[test]
    fn snapshot_prevents_multi_hop_and_full_recipient_is_skipped() {
        let mut world = world();
        let own = faction(&mut world, 0);
        tower(&mut world, own, 0, 0.0);
        let a = minion(&mut world, own, 5, 10.0, 0);
        let b = minion(&mut world, own, 6, 20.0, 0);
        reconcile_links(&mut world);
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (1, 0));
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (1, 1));
        world.get_mut::<EnergyStore>(a).unwrap().current = 5;
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (4, 2));
    }
    #[test]
    fn minion_transfer_conserves_energy_and_can_spend_last_unit() {
        let mut world = world();
        let own = faction(&mut world, 0);
        let root = tower(&mut world, own, 0, 0.0);
        let a = minion(&mut world, own, 5, 10.0, 1);
        let b = minion(&mut world, own, 6, 20.0, 0);
        reconcile_links(&mut world);
        // Isolate the finite sender's contribution while retaining its live root.
        for (entity, link) in links(&mut world) {
            if link.source == root {
                world.despawn(entity);
            }
        }
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (0, 1));
        transfer_pulse(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (0, 1));
    }
    #[test]
    fn timing_accumulates_scaled_time_and_processes_due_pulses_sequentially() {
        let mut world = world();
        let own = faction(&mut world, 0);
        tower(&mut world, own, 0, 0.0);
        let a = minion(&mut world, own, 5, 10.0, 0);
        let b = minion(&mut world, own, 6, 20.0, 0);
        reconcile_links(&mut world);
        world
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs(1));
        for _ in 0..4 {
            transfer_energy(&mut world);
        }
        assert_eq!(world.resource::<EnergyClock>().pulses, 0);
        transfer_energy(&mut world);
        assert_eq!((energy(&world, a), energy(&world, b)), (1, 0));
        world.resource_mut::<MatchState>().speed = 4.0;
        world
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs(3));
        transfer_energy(&mut world);
        assert_eq!(world.resource::<EnergyClock>().pulses, 3);
        assert_eq!(world.resource::<EnergyClock>().elapsed, 2.0);
        assert_eq!((energy(&world, a), energy(&world, b)), (1, 2));
    }
}
