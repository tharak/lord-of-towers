use crate::components::{
    Allegiance, DragControl, ElementDeposit, ElementInventory, GatheringState, Minion, Movement,
    Position, SimulationId, Tower, Trail, Vision, VisitedTrailMarkers,
};
use crate::entities::trail::marker;
use crate::resources::{GatheringRuntime, MatchState, SimulationConfig};
use crate::scheduling::simulation_delta;
use bevy::prelude::*;

pub(crate) fn stable(world: &World, entity: Entity) -> u64 {
    world
        .get::<SimulationId>(entity)
        .map_or(entity.to_bits(), |id| id.0)
}
pub(crate) fn units(world: &mut World) -> Vec<Entity> {
    let mut result: Vec<_> = world
        .query_filtered::<Entity, With<Minion>>()
        .iter(world)
        .collect();
    result.sort_by_key(|e| stable(world, *e));
    result
}
pub(crate) fn nearest_tower(world: &mut World, position: Vec2, faction: Entity) -> Option<Entity> {
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
pub(crate) fn explore(world: &mut World, entity: Entity, position: Vec2, dt: f32, force: bool) {
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
pub(crate) fn navigate(world: &mut World) {
    let dt = simulation_delta(
        world.resource::<Time<Fixed>>(),
        world.resource::<MatchState>(),
    );
    for entity in units(world) {
        if world.get::<DragControl>(entity).is_some() {
            continue;
        }
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
