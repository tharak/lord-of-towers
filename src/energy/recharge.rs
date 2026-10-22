use crate::model::*;
use bevy::prelude::*;

pub(super) fn recharge_energy(world: &mut World) {
    let state = world.resource::<MatchState>();
    if state.paused || state.browser_hidden || state.winner.is_some() {
        return;
    }
    let interval = {
        let config = world.resource::<SimulationConfig>();
        config.energy_tick_seconds * config.energy_recharge_ticks as f32
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
        recharge_pulse(world);
    }
}

/// A pulse charges each living minion at most once, regardless of area overlap.
fn recharge_pulse(world: &mut World) {
    let towers: Vec<_> = world
        .query_filtered::<(&Position, &Allegiance, &InfluenceEmitter, &Health), With<Tower>>()
        .iter(world)
        .filter(|(_, _, area, health)| {
            health.current > 0.0 && area.radius.is_finite() && area.radius >= 0.0
        })
        .map(|(position, faction, area, _)| (position.0, *faction, area.radius * area.radius))
        .collect();
    let mut minions =
        world.query_filtered::<(&Position, &Allegiance, &Health, &mut EnergyStore), With<Minion>>();
    for (position, faction, health, mut energy) in minions.iter_mut(world) {
        if health.current > 0.0
            && energy.current < energy.capacity
            && towers.iter().any(|(center, owner, radius_squared)| {
                owner == faction && center.distance_squared(position.0) <= *radius_squared
            })
        {
            energy.current += 1;
        }
    }
}
