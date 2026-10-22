//! Local, renderer-independent rawElement gathering.
use crate::model::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct GatheringPlugin;
impl Plugin for GatheringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GatheringRuntime>()
            .init_resource::<TrailIds>()
            .add_systems(FixedUpdate, navigate.in_set(SimSet::Target))
            .add_systems(
                FixedUpdate,
                (decay, collect, deliver, emit)
                    .chain()
                    .in_set(SimSet::Gathering),
            );
    }
}
#[derive(Resource)]
struct GatheringRuntime {
    random: u64,
    next_drop: u64,
}
impl Default for GatheringRuntime {
    fn default() -> Self {
        Self {
            random: 42,
            next_drop: 0,
        }
    }
}
impl GatheringRuntime {
    fn unit(&mut self) -> f32 {
        self.random = self.random.wrapping_add(0x9e3779b97f4a7c15);
        let mut n = self.random;
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((n ^ (n >> 31)) >> 40) as f32 / 16_777_216.0
    }
}
/// Lowest arrived sequence per trip prevents joining an exhausted trail again.
#[derive(Component, Default)]
struct VisitedTrails(HashMap<u64, u32>);

pub fn spawn_deposits(commands: &mut Commands, config: &SimulationConfig) {
    let mut runtime = GatheringRuntime {
        random: config.seed,
        next_drop: 0,
    };
    let mut total = 0;
    let mut positions: Vec<Vec2> = Vec::new();
    let towers: Vec<_> = (0..5)
        .map(|i| {
            Vec2::from_angle(i as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2)
                * 32.0_f32.min(config.half_extent * 0.75)
        })
        .collect();
    for index in 0..config.deposit_count {
        let extent = (config.half_extent - 2.0).max(0.0);
        let mut position = Vec2::ZERO;
        for _ in 0..512 {
            position = Vec2::new(runtime.unit() * 2.0 - 1.0, runtime.unit() * 2.0 - 1.0) * extent;
            if towers.iter().all(|p| p.distance_squared(position) >= 25.0)
                && positions
                    .iter()
                    .all(|p| p.distance_squared(position) >= 9.0)
            {
                break;
            }
        }
        positions.push(position);
        let amount = 5 + (runtime.unit() * 6.0) as u32;
        total += amount;
        commands.spawn((
            Name::new("rawElement deposit"),
            MatchEntity,
            EntityKind::RawElementDeposit,
            RawElementDeposit,
            Position(position),
            RawElement(amount),
            SimulationId((1 << 63) + index as u64),
        ));
    }
    commands.insert_resource(runtime);
    commands.insert_resource(TrailIds::default());
    commands.insert_resource(DrawTrailRequests::default());
    commands.insert_resource(EraseTrailRequests::default());
    commands.insert_resource(RawElementEconomy { initial: total });
}
pub fn drop_cargo(commands: &mut Commands, position: Vec2, amount: u32) {
    if amount > 0 {
        commands.queue(move |world: &mut World| {
            let id = {
                let mut runtime = world.resource_mut::<GatheringRuntime>();
                let id = (1_u64 << 62) + runtime.next_drop;
                runtime.next_drop += 1;
                id
            };
            world.spawn((
                Name::new("Dropped rawElement"),
                MatchEntity,
                EntityKind::RawElementDeposit,
                RawElementDeposit,
                Position(position),
                RawElement(amount),
                SimulationId(id),
            ));
        });
    }
}
fn stable(world: &World, entity: Entity) -> u64 {
    world
        .get::<SimulationId>(entity)
        .map_or(entity.to_bits(), |id| id.0)
}
fn units(world: &mut World) -> Vec<Entity> {
    let mut result: Vec<_> = world
        .query_filtered::<Entity, With<Minion>>()
        .iter(world)
        .collect();
    result.sort_by_key(|e| stable(world, *e));
    result
}
fn nearest_tower(world: &mut World, position: Vec2, faction: Entity) -> Option<Entity> {
    let mut candidates: Vec<_> = world
        .query_filtered::<(Entity, &Position, &Allegiance), With<Tower>>()
        .iter(world)
        .filter(|(_, _, a)| a.0 == faction)
        .map(|(e, p, _)| (e, position.distance_squared(p.0)))
        .collect();
    candidates.sort_by(|a, b| {
        a.1.total_cmp(&b.1)
            .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
    });
    candidates.first().map(|c| c.0)
}
fn explore(world: &mut World, entity: Entity, position: Vec2, dt: f32, force: bool) {
    let Some(movement) = world.get::<Movement>(entity).copied() else {
        return;
    };
    if force || movement.retarget_in <= dt || position.distance_squared(movement.target) <= 1.0 {
        let extent = world.resource::<SimulationConfig>().half_extent;
        let mut rng = world.resource_mut::<GatheringRuntime>();
        let angle = rng.unit() * std::f32::consts::TAU;
        let distance = 2.0 + rng.unit() * 8.0;
        let target = (position + Vec2::from_angle(angle) * distance)
            .clamp(Vec2::splat(-extent), Vec2::splat(extent));
        let mut movement = world.get_mut::<Movement>(entity).unwrap();
        movement.target = target;
        movement.retarget_in = 3.0;
    } else {
        world.get_mut::<Movement>(entity).unwrap().retarget_in -= dt;
    }
}
fn navigate(world: &mut World) {
    let dt = simulation_delta(
        world.resource::<Time<Fixed>>(),
        world.resource::<MatchState>(),
    );
    for entity in units(world) {
        let (Some(position), Some(owner), Some(vision), Some(state), Some(cargo)) = (
            world.get::<Position>(entity).copied(),
            world.get::<Allegiance>(entity).copied(),
            world.get::<Vision>(entity).copied(),
            world.get::<GatheringState>(entity).copied(),
            world.get::<RawElement>(entity).copied(),
        ) else {
            continue;
        };
        if world.get::<Movement>(entity).is_none() {
            continue;
        }
        if cargo.0 > 0 {
            let previous = match state {
                GatheringState::Returning { tower } => tower,
                _ => None,
            };
            let tower = previous
                .filter(|t| {
                    world.get::<Tower>(*t).is_some() && world.get::<Allegiance>(*t) == Some(&owner)
                })
                .or_else(|| nearest_tower(world, position.0, owner.0));
            world
                .entity_mut(entity)
                .insert(GatheringState::Returning { tower });
            if let Some(target) = tower.and_then(|t| world.get::<Position>(t).map(|p| p.0)) {
                world.get_mut::<Movement>(entity).unwrap().target = target;
            } else {
                explore(world, entity, position.0, dt, false);
            }
            continue;
        }
        let mut deposits: Vec<_> = world
            .query_filtered::<(Entity, &Position, &RawElement), With<RawElementDeposit>>()
            .iter(world)
            .filter_map(|(e, p, a)| {
                let d = position.0.distance_squared(p.0);
                (d <= vision.radius * vision.radius && a.0 > 0).then_some((e, p.0, d))
            })
            .collect();
        deposits.sort_by(|a, b| {
            a.2.total_cmp(&b.2)
                .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
        });
        if let Some((_, target, _)) = deposits.first() {
            world.entity_mut(entity).insert(GatheringState::Exploring);
            let mut movement = world.get_mut::<Movement>(entity).unwrap();
            movement.target = *target;
            // If sight is lost next step, explore immediately instead of pursuing hidden cargo.
            movement.retarget_in = 0.0;
            continue;
        }
        let mut trails: Vec<_> = world
            .query::<(Entity, &Position, &Allegiance, &Trail)>()
            .iter(world)
            .filter_map(|(e, p, a, t)| {
                let d = position.0.distance_squared(p.0);
                (a.0 == owner.0 && t.remaining > 0.0 && d <= vision.radius * vision.radius)
                    .then_some((e, p.0, *t, d))
            })
            .collect();
        let mut following = None;
        if let GatheringState::Following { trip, sequence } = state {
            if let Some(marker) = trails
                .iter()
                .find(|(_, _, t, _)| t.trip == trip && t.sequence == sequence)
            {
                if marker.3 > 1.0 {
                    following = Some(*marker)
                } else {
                    if world.get::<VisitedTrails>(entity).is_none() {
                        world.entity_mut(entity).insert(VisitedTrails::default());
                    }
                    world
                        .get_mut::<VisitedTrails>(entity)
                        .unwrap()
                        .0
                        .entry(trip)
                        .and_modify(|s| *s = (*s).min(sequence))
                        .or_insert(sequence);
                }
            }
            if following.is_none() {
                trails.retain(|(_, _, t, _)| t.trip == trip && t.sequence < sequence);
            }
        }
        if following.is_none() {
            trails.retain(|(_, _, t, _)| {
                world
                    .get::<VisitedTrails>(entity)
                    .and_then(|v| v.0.get(&t.trip))
                    .is_none_or(|s| t.sequence < *s)
            });
            trails.sort_by(|a, b| {
                a.3.total_cmp(&b.3)
                    .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
            });
            following = trails.first().copied();
        }
        if let Some((_, target, t, _)) = following {
            world.entity_mut(entity).insert(GatheringState::Following {
                trip: t.trip,
                sequence: t.sequence,
            });
            world.get_mut::<Movement>(entity).unwrap().target = target;
        } else {
            world.entity_mut(entity).insert(GatheringState::Exploring);
            explore(
                world,
                entity,
                position.0,
                dt,
                !matches!(state, GatheringState::Exploring),
            );
        }
    }
}
fn decay(world: &mut World) {
    let dt = simulation_delta(
        world.resource::<Time<Fixed>>(),
        world.resource::<MatchState>(),
    );
    let mut expired = Vec::new();
    for (e, mut trail) in world.query::<(Entity, &mut Trail)>().iter_mut(world) {
        trail.remaining -= dt;
        if trail.remaining <= 0.0 {
            expired.push(e);
        }
    }
    for e in expired {
        world.despawn(e);
    }
    clean_trail_memory(world);
}

/// Shared allocation prevents player strokes and cargo paths from becoming one trip.
pub(crate) fn allocate_trip(world: &mut World) -> u64 {
    let mut ids = world.resource_mut::<TrailIds>();
    let trip = ids.next_trip;
    ids.next_trip += 1;
    trip
}

pub(crate) fn clean_trail_memory(world: &mut World) {
    let active: HashSet<_> = world
        .query::<&Trail>()
        .iter(world)
        .map(|t| t.trip)
        .collect();
    for mut visited in world.query::<&mut VisitedTrails>().iter_mut(world) {
        visited.0.retain(|trip, _| active.contains(trip));
    }
    let live: HashSet<_> = world
        .query::<&Trail>()
        .iter(world)
        .map(|t| (t.trip, t.sequence))
        .collect();
    for (mut state, mut movement) in world
        .query::<(&mut GatheringState, &mut Movement)>()
        .iter_mut(world)
    {
        if let GatheringState::Following { trip, sequence } = *state {
            if !live.contains(&(trip, sequence)) {
                *state = GatheringState::Exploring;
                movement.retarget_in = 0.0;
            }
        }
    }
}
pub(crate) fn marker(
    world: &mut World,
    position: Vec2,
    owner: Allegiance,
    trip: u64,
    sequence: u32,
) -> Entity {
    let remaining = world.resource::<SimulationConfig>().trail_lifetime;
    world
        .spawn((
            Name::new("rawElement trail"),
            MatchEntity,
            EntityKind::Trail,
            Position(position),
            owner,
            Trail {
                trip,
                sequence,
                remaining,
            },
        ))
        .id()
}
fn collect(world: &mut World) {
    let radius = world.resource::<SimulationConfig>().pickup_radius;
    let mut deposits: Vec<_> = world
        .query_filtered::<Entity, With<RawElementDeposit>>()
        .iter(world)
        .collect();
    deposits.sort_by_key(|e| stable(world, *e));
    for deposit in deposits {
        let Some(position) = world.get::<Position>(deposit).copied() else {
            continue;
        };
        let mut candidates: Vec<_> = world
            .query_filtered::<(Entity, &Position, &RawElement, &Vision), With<Minion>>()
            .iter(world)
            .filter_map(|(e, p, a, v)| {
                let d = p.0.distance_squared(position.0);
                (a.0 == 0 && d <= radius * radius && d <= v.radius * v.radius).then_some((e, d))
            })
            .collect();
        candidates.sort_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
        });
        for (entity, _) in candidates {
            let Some(amount) = world.get::<RawElement>(deposit).copied() else {
                break;
            };
            if amount.0 == 0 {
                break;
            }
            let Some(owner) = world.get::<Allegiance>(entity).copied() else {
                continue;
            };
            let p = world.get::<Position>(entity).unwrap().0;
            let tower = nearest_tower(world, p, owner.0);
            let trip = allocate_trip(world);
            world.entity_mut(entity).insert((
                RawElement(1),
                GatheringState::Returning { tower },
                TrailEmitter {
                    trip,
                    next_sequence: 1,
                    last_position: p,
                },
            ));
            marker(world, p, owner, trip, 0);
            world.get_mut::<RawElement>(deposit).unwrap().0 -= 1;
            if amount.0 == 1 {
                world.despawn(deposit);
                break;
            }
        }
    }
}
fn deliver(world: &mut World) {
    let radius = world.resource::<SimulationConfig>().delivery_radius;
    for entity in units(world) {
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
            world.get::<RawElement>(entity),
        ) else {
            continue;
        };
        if cargo.0 == 0 || a != ta || p.0.distance_squared(tp.0) > radius * radius {
            continue;
        }
        let amount = cargo.0;
        if let Some(mut stock) = world.get_mut::<RawElement>(tower) {
            stock.0 += amount;
        } else {
            continue;
        }
        world
            .entity_mut(entity)
            .insert((RawElement(0), GatheringState::Exploring))
            .remove::<TrailEmitter>();
        if let Some(mut movement) = world.get_mut::<Movement>(entity) {
            movement.retarget_in = 0.0;
        }
    }
}
fn emit(world: &mut World) {
    let spacing = world.resource::<SimulationConfig>().trail_spacing;
    let emissions: Vec<_> = world
        .query::<(Entity, &Position, &Allegiance, &TrailEmitter)>()
        .iter(world)
        .filter(|(_, p, _, e)| p.0.distance_squared(e.last_position) >= spacing * spacing)
        .map(|(e, p, a, t)| (e, p.0, *a, *t))
        .collect();
    for (entity, p, owner, emitter) in emissions {
        marker(world, p, owner, emitter.trip, emitter.next_sequence);
        let mut emitter = world.get_mut::<TrailEmitter>(entity).unwrap();
        emitter.next_sequence += 1;
        emitter.last_position = p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationConfig::default());
        world.insert_resource(MatchState::default());
        world.insert_resource(GatheringRuntime::default());
        world.init_resource::<TrailIds>();
        world.insert_resource(Time::<Fixed>::from_hz(30.0));
        world
    }
    fn unit(world: &mut World, faction: Entity, id: u64, position: Vec2) -> Entity {
        world
            .spawn((
                Minion,
                SimulationId(id),
                Position(position),
                Allegiance(faction),
                RawElement(0),
                Vision { radius: 15.0 },
                GatheringState::Exploring,
                EnergyStore {
                    current: 0,
                    capacity: 5,
                },
                Movement {
                    speed: 4.0,
                    target: position,
                    retarget_in: 0.0,
                },
            ))
            .id()
    }
    fn deposit(world: &mut World, position: Vec2, amount: u32) -> Entity {
        world
            .spawn((RawElementDeposit, Position(position), RawElement(amount)))
            .id()
    }
    fn target(world: &World, unit: Entity) -> Vec2 {
        world.get::<Movement>(unit).unwrap().target
    }
    fn tower(world: &mut World, faction: Entity, position: Vec2) -> Entity {
        world
            .spawn((
                Tower { slot: 0 },
                Allegiance(faction),
                Position(position),
                RawElement(0),
            ))
            .id()
    }
    #[test]
    fn vision_is_inclusive_and_lost_deposits_are_not_tracked() {
        let mut w = world();
        let faction = w.spawn_empty().id();
        let u = unit(&mut w, faction, 1, Vec2::ZERO);
        let d = deposit(&mut w, Vec2::X * 15.0, 5);
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 15.0);
        // No movement energy is needed; the same minion has zero energy.
        w.get_mut::<Position>(d).unwrap().0 = Vec2::X * 15.01;
        navigate(&mut w);
        assert!(target(&w, u).length() <= 10.001);
        assert_ne!(target(&w, u), Vec2::X * 15.0);
        w.get_mut::<Vision>(u).unwrap().radius = 2.0;
        w.get_mut::<Position>(d).unwrap().0 = Vec2::X * 2.0;
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 2.0);
    }
    #[test]
    fn trails_are_friendly_visible_and_followed_toward_lower_sequences() {
        let mut w = world();
        let own = w.spawn_empty().id();
        let enemy = w.spawn_empty().id();
        let u = unit(&mut w, own, 1, Vec2::ZERO);
        marker(&mut w, Vec2::X, Allegiance(enemy), 90, 0);
        marker(&mut w, Vec2::X * 16.0, Allegiance(own), 91, 0);
        navigate(&mut w);
        assert!(matches!(
            w.get::<GatheringState>(u).unwrap(),
            GatheringState::Exploring
        ));
        marker(&mut w, Vec2::X * 2.0, Allegiance(own), 1, 2);
        marker(&mut w, Vec2::X * 4.0, Allegiance(own), 1, 1);
        marker(&mut w, Vec2::X * 6.0, Allegiance(own), 1, 0);
        for (x, seq) in [(2.0, 2), (4.0, 1), (6.0, 0)] {
            navigate(&mut w);
            assert_eq!(target(&w, u), Vec2::X * x);
            assert!(
                matches!(w.get::<GatheringState>(u).unwrap(), GatheringState::Following { trip:1, sequence } if *sequence == seq)
            );
            w.get_mut::<Position>(u).unwrap().0 = Vec2::X * x;
        }
        navigate(&mut w);
        assert!(matches!(
            w.get::<GatheringState>(u).unwrap(),
            GatheringState::Exploring
        ));
        // Leave unrelated markers outside sight; exhausted trip cannot cause a loop.
        w.get_mut::<Vision>(u).unwrap().radius = 7.0;
        navigate(&mut w);
        assert!(matches!(
            w.get::<GatheringState>(u).unwrap(),
            GatheringState::Exploring
        ));
    }
    #[test]
    fn visible_deposit_overrides_trail_and_invisible_continuation_is_rejected() {
        let mut w = world();
        let own = w.spawn_empty().id();
        let u = unit(&mut w, own, 1, Vec2::ZERO);
        marker(&mut w, Vec2::X * 2.0, Allegiance(own), 1, 1);
        marker(&mut w, Vec2::X * 30.0, Allegiance(own), 1, 0);
        let d = deposit(&mut w, Vec2::Y * 10.0, 5);
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::Y * 10.0);
        w.despawn(d);
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 2.0);
        w.get_mut::<Position>(u).unwrap().0 = Vec2::X * 2.0;
        navigate(&mut w);
        assert!(matches!(
            w.get::<GatheringState>(u).unwrap(),
            GatheringState::Exploring
        ));
        assert!(target(&w, u).distance(Vec2::X * 2.0) <= 10.001);
    }
    #[test]
    fn collection_ties_are_stable_capacity_one_and_depletion_is_permanent() {
        let mut w = world();
        let own = w.spawn_empty().id();
        let larger = unit(&mut w, own, 9, Vec2::X * 0.5);
        let smaller = unit(&mut w, own, 7, -Vec2::X * 0.5);
        let d = deposit(&mut w, Vec2::ZERO, 1);
        collect(&mut w);
        assert!(w.get_entity(d).is_err());
        assert_eq!(w.get::<RawElement>(smaller).unwrap().0, 1);
        assert_eq!(w.get::<RawElement>(larger).unwrap().0, 0);
        let other = deposit(&mut w, Vec2::ZERO, 5);
        collect(&mut w);
        assert_eq!(w.get::<RawElement>(smaller).unwrap().0, 1);
        assert_eq!(w.get::<RawElement>(larger).unwrap().0, 1);
        assert_eq!(w.get::<RawElement>(other).unwrap().0, 4);
        collect(&mut w);
        assert_eq!(w.get::<RawElement>(other).unwrap().0, 4);
    }
    #[test]
    fn delivery_knows_remote_towers_and_reselects_after_capture() {
        let mut w = world();
        let own = w.spawn_empty().id();
        let enemy = w.spawn_empty().id();
        let near = tower(&mut w, own, Vec2::X * 20.0);
        let far = tower(&mut w, own, Vec2::X * 30.0);
        let u = unit(&mut w, own, 1, Vec2::ZERO);
        deposit(&mut w, Vec2::ZERO, 1);
        collect(&mut w);
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 20.0);
        w.entity_mut(near).insert(Allegiance(enemy));
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 30.0);
        w.get_mut::<Position>(u).unwrap().0 = Vec2::X * 27.0;
        deliver(&mut w);
        assert_eq!(w.get::<RawElement>(far).unwrap().0, 1);
        assert_eq!(w.get::<RawElement>(u).unwrap().0, 0);
        assert!(w.get::<TrailEmitter>(u).is_none());
        assert_eq!(w.query::<&Trail>().iter(&w).count(), 1);
        w.entity_mut(u)
            .insert((RawElement(1), GatheringState::Returning { tower: None }));
        w.entity_mut(far).insert(Allegiance(enemy));
        navigate(&mut w);
        deliver(&mut w);
        assert_eq!(w.get::<RawElement>(u).unwrap().0, 1);
    }
    #[test]
    fn emission_spacing_and_decay_use_simulation_speed() {
        let mut w = world();
        let own = w.spawn_empty().id();
        let u = unit(&mut w, own, 1, Vec2::ZERO);
        deposit(&mut w, Vec2::ZERO, 1);
        collect(&mut w);
        w.get_mut::<Position>(u).unwrap().0 = Vec2::X * 1.99;
        emit(&mut w);
        assert_eq!(w.query::<&Trail>().iter(&w).count(), 1);
        w.get_mut::<Position>(u).unwrap().0 = Vec2::X * 2.0;
        emit(&mut w);
        assert_eq!(w.query::<&Trail>().iter(&w).count(), 2);
        w.resource_mut::<MatchState>().speed = 4.0;
        w.resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs(7));
        decay(&mut w);
        assert!(w.query::<&Trail>().iter(&w).all(|t| t.remaining == 2.0));
        w.resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_millis(500));
        decay(&mut w);
        assert_eq!(w.query::<&Trail>().iter(&w).count(), 0);
    }
    #[test]
    fn player_markers_follow_with_gaps_and_cargo_keeps_returning() {
        let mut w = world();
        let own = w.spawn_empty().id();
        let home = tower(&mut w, own, Vec2::Y * 20.0);
        let u = unit(&mut w, own, 1, Vec2::ZERO);
        let trip = allocate_trip(&mut w);
        let first = marker(&mut w, Vec2::X * 2.0, Allegiance(own), trip, 7);
        w.entity_mut(first).insert(PlayerDrawnTrail { serial: 0 });
        let last = marker(&mut w, Vec2::X * 6.0, Allegiance(own), trip, 0);
        w.entity_mut(last).insert(PlayerDrawnTrail { serial: 1 });
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 2.0);
        w.get_mut::<Position>(u).unwrap().0 = Vec2::X * 2.0;
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::X * 6.0);
        w.entity_mut(u).insert((
            RawElement(1),
            GatheringState::Returning { tower: Some(home) },
        ));
        navigate(&mut w);
        assert_eq!(target(&w, u), Vec2::Y * 20.0);
        w.entity_mut(u).insert((
            RawElement(0),
            GatheringState::Following { trip, sequence: 0 },
            VisitedTrails(HashMap::from([(trip, 7)])),
        ));
        w.despawn(last);
        clean_trail_memory(&mut w);
        assert!(matches!(
            w.get::<GatheringState>(u),
            Some(GatheringState::Exploring)
        ));
        assert_eq!(w.get::<Movement>(u).unwrap().retarget_in, 0.0);
        w.despawn(first);
        clean_trail_memory(&mut w);
        assert!(w.get::<VisitedTrails>(u).unwrap().0.is_empty());
    }
    #[test]
    fn match_setup_resets_shared_identity_and_pending_gestures() {
        let mut w = world();
        w.insert_resource(DrawTrailRequests(vec![vec![Vec2::ZERO]]));
        w.insert_resource(EraseTrailRequests(vec![Vec2::ZERO]));
        w.resource_mut::<TrailIds>().next_trip = 99;
        w.resource_mut::<TrailIds>().next_player_marker = 88;
        let config = SimulationConfig {
            deposit_count: 0,
            ..default()
        };
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &w);
            spawn_deposits(&mut commands, &config);
        }
        queue.apply(&mut w);
        assert_eq!(w.resource::<TrailIds>().next_trip, 0);
        assert_eq!(w.resource::<TrailIds>().next_player_marker, 0);
        assert!(w.resource::<DrawTrailRequests>().0.is_empty());
        assert!(w.resource::<EraseTrailRequests>().0.is_empty());
    }
}
