//! Local, renderer-independent Element gathering.
use crate::model::*;
use bevy::prelude::*;
use std::collections::HashSet;

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
/// Reached and bypassed marker identities prevent loops without imposing a trip direction.
#[derive(Component, Default)]
struct VisitedTrailMarkers(HashSet<Entity>);

pub fn spawn_deposits(commands: &mut Commands, config: &SimulationConfig) {
    let mut runtime = GatheringRuntime {
        random: config.seed,
        next_drop: 0,
    };
    let mut total = [0; 5];
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
        let element = Element::ALL[index % Element::ALL.len()];
        total[element as usize] += amount;
        commands.spawn((
            Name::new(format!("{} deposit", element.name())),
            MatchEntity,
            EntityKind::ElementDeposit,
            ElementDeposit,
            Position(position),
            ElementInventory::homogeneous(element, amount),
            SimulationId((1 << 63) + index as u64),
        ));
    }
    commands.insert_resource(runtime);
    commands.insert_resource(TrailIds::default());
    commands.insert_resource(ElementEconomy {
        initial: total,
        consumed: [0; 5],
    });
}
/// Death drops split mixed stock into homogeneous deposits without changing totals.
pub fn drop_cargo(commands: &mut Commands, position: Vec2, inventory: ElementInventory) {
    let counts = inventory.counts();
    if inventory.is_empty() {
        return;
    }
    commands.queue(move |world: &mut World| {
        for element in Element::ALL {
            let amount = counts[element as usize];
            if amount == 0 {
                continue;
            }
            let id = {
                let mut runtime = world.resource_mut::<GatheringRuntime>();
                let id = (1_u64 << 62) + runtime.next_drop;
                runtime.next_drop += 1;
                id
            };
            world.spawn((
                Name::new(format!("Dropped {}", element.name())),
                MatchEntity,
                EntityKind::ElementDeposit,
                ElementDeposit,
                Position(position),
                ElementInventory::homogeneous(element, amount),
                SimulationId(id),
            ));
        }
    });
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
        if world.get::<DragControl>(entity).is_some() { continue; }
        let (Some(position), Some(owner), Some(vision), Some(state), Some(cargo)) = (
            world.get::<Position>(entity).copied(),
            world.get::<Allegiance>(entity).copied(),
            world.get::<Vision>(entity).copied(),
            world.get::<GatheringState>(entity).copied(),
            world.get::<ElementInventory>(entity).cloned(),
        ) else {
            continue;
        };
        if world.get::<Movement>(entity).is_none() {
            continue;
        }
        if !cargo.is_empty() {
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
            .query_filtered::<(Entity, &Position, &ElementInventory), With<ElementDeposit>>()
            .iter(world)
            .filter_map(|(e, p, a)| {
                let d = position.0.distance_squared(p.0);
                (d <= vision.radius * vision.radius && !a.is_empty()).then_some((e, p.0, d))
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
        let visible_markers: Vec<_> = trails.iter().map(|(marker, ..)| *marker).collect();
        trails.retain(|(marker, _, _, _)| {
            world
                .get::<VisitedTrailMarkers>(entity)
                .is_none_or(|visited| !visited.0.contains(marker))
        });
        trails.sort_by(|a, b| {
            b.2.remaining
                .total_cmp(&a.2.remaining)
                .then_with(|| a.3.total_cmp(&b.3))
                .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
        });
        // Reconsider freshness every tick, including while already following a marker.
        // Consume arrived candidates immediately so the next target is chosen this tick.
        let mut following = None;
        for marker in trails {
            if marker.3 <= 1.0 {
                if world.get::<VisitedTrailMarkers>(entity).is_none() {
                    world
                        .entity_mut(entity)
                        .insert(VisitedTrailMarkers::default());
                }
                world
                    .get_mut::<VisitedTrailMarkers>(entity)
                    .unwrap()
                    .0
                    .insert(marker.0);
            } else {
                following = Some(marker);
                break;
            }
        }
        if let Some((selected, target, t, _)) = following {
            // Keep the chosen destination eligible until arrival; bypass its visible neighbors.
            if world.get::<VisitedTrailMarkers>(entity).is_none() {
                world
                    .entity_mut(entity)
                    .insert(VisitedTrailMarkers::default());
            }
            world
                .get_mut::<VisitedTrailMarkers>(entity)
                .unwrap()
                .0
                .extend(
                    visible_markers
                        .into_iter()
                        .filter(|marker| *marker != selected),
                );
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
        .query_filtered::<Entity, With<Trail>>()
        .iter(world)
        .collect();
    for mut visited in world.query::<&mut VisitedTrailMarkers>().iter_mut(world) {
        visited.0.retain(|marker| active.contains(marker));
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
        if let GatheringState::Following { trip, sequence } = *state
            && !live.contains(&(trip, sequence))
        {
            *state = GatheringState::Exploring;
            movement.retarget_in = 0.0;
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
            Name::new("Element trail"),
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
/// Carriers renew nearby friendly trails without changing their identity or path.
fn carrier_marker(
    world: &mut World,
    position: Vec2,
    owner: Allegiance,
    trip: u64,
    sequence: u32,
) -> Entity {
    let config = world.resource::<SimulationConfig>();
    let radius_squared = config.trail_spacing * config.trail_spacing;
    let lifetime = config.trail_lifetime;
    let mut candidates: Vec<_> = world
        .query::<(Entity, &Position, &Allegiance, &Trail)>()
        .iter(world)
        .filter_map(|(entity, p, allegiance, trail)| {
            let distance = position.distance_squared(p.0);
            (*allegiance == owner && trail.remaining > 0.0 && distance <= radius_squared)
                .then_some((entity, distance))
        })
        .collect();
    candidates.sort_by(|a, b| {
        a.1.total_cmp(&b.1)
            .then_with(|| stable(world, a.0).cmp(&stable(world, b.0)))
    });
    if let Some(&(entity, _)) = candidates.first() {
        world.get_mut::<Trail>(entity).unwrap().remaining = lifetime;
        entity
    } else {
        marker(world, position, owner, trip, sequence)
    }
}

fn collect(world: &mut World) {
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
fn deliver(world: &mut World) {
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
fn emit(world: &mut World) {
    let spacing = world.resource::<SimulationConfig>().trail_spacing;
    let emissions: Vec<_> = world
        .query::<(Entity, &Position, &Allegiance, &TrailEmitter)>()
        .iter(world)
        .filter(|(_, p, _, e)| p.0.distance_squared(e.last_position) >= spacing * spacing)
        .map(|(e, p, a, t)| (e, p.0, *a, *t))
        .collect();
    for (entity, p, owner, emitter) in emissions {
        carrier_marker(world, p, owner, emitter.trip, emitter.next_sequence);
        let mut emitter = world.get_mut::<TrailEmitter>(entity).unwrap();
        emitter.next_sequence += 1;
        emitter.last_position = p;
    }
}
