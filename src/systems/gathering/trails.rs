use crate::components::{
    Allegiance, GatheringState, Movement, Position, Trail, TrailEmitter, VisitedTrailMarkers,
};
use crate::entities::trail::marker;
use crate::resources::{MatchState, SimulationConfig, TrailIds};
use crate::scheduling::simulation_delta;
use crate::systems::gathering::navigation::stable;
use bevy::prelude::*;
use std::collections::HashSet;

pub(crate) fn decay(world: &mut World) {
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
/// Carriers renew nearby friendly trails without changing their identity or path.
pub(crate) fn carrier_marker(
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
pub(crate) fn emit(world: &mut World) {
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
