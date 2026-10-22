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
    fn tower(world: &mut World, faction: Entity, x: f32) -> Entity {
        world
            .spawn((
                Tower { slot: 0 },
                Position(Vec2::new(x, 0.0)),
                Allegiance(faction),
                InfluenceEmitter {
                    radius: 15.0,
                    strength: 4.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id()
    }
    fn minion(world: &mut World, faction: Entity, position: Vec2, current: u8) -> Entity {
        world
            .spawn((
                Minion,
                Position(position),
                Allegiance(faction),
                Health {
                    current: 30.0,
                    max: 30.0,
                },
                EnergyStore {
                    current,
                    capacity: 5,
                },
            ))
            .id()
    }
    fn energy(world: &World, entity: Entity) -> u8 {
        world.get::<EnergyStore>(entity).unwrap().current
    }
    #[test]
    fn inclusive_circle_overlap_factions_and_capacity() {
        let mut world = world();
        let own = world.spawn(Faction { index: 0 }).id();
        let enemy = world.spawn(Faction { index: 1 }).id();
        tower(&mut world, own, 0.0);
        tower(&mut world, own, 0.0);
        // Enemy presence and contested influence cannot suppress friendly recharge.
        tower(&mut world, enemy, 0.0);
        let border = minion(&mut world, own, Vec2::new(9.0, 12.0), 0);
        let outside = minion(&mut world, own, Vec2::new(15.001, 0.0), 0);
        let diagonal = minion(&mut world, own, Vec2::splat(11.0), 0);
        let full = minion(&mut world, own, Vec2::ZERO, 5);
        let hostile = minion(&mut world, enemy, Vec2::new(16.0, 0.0), 0);
        recharge_pulse(&mut world);
        assert_eq!(energy(&world, border), 1);
        for entity in [outside, diagonal, hostile] {
            assert_eq!(energy(&world, entity), 0);
        }
        assert_eq!(energy(&world, full), 5);
        for _ in 0..10 {
            recharge_pulse(&mut world);
        }
        assert_eq!(energy(&world, border), 5);
    }
    #[test]
    fn death_movement_and_despawn_remove_recharge_immediately() {
        let mut world = world();
        let own = world.spawn(Faction { index: 0 }).id();
        let root = tower(&mut world, own, 0.0);
        let unit = minion(&mut world, own, Vec2::ZERO, 0);
        world.get_mut::<Health>(root).unwrap().current = 0.0;
        recharge_pulse(&mut world);
        assert_eq!(energy(&world, unit), 0);
        world.get_mut::<Health>(root).unwrap().current = 100.0;
        world.get_mut::<Health>(unit).unwrap().current = 0.0;
        recharge_pulse(&mut world);
        assert_eq!(energy(&world, unit), 0);
        world.get_mut::<Health>(unit).unwrap().current = 30.0;
        world.get_mut::<Position>(unit).unwrap().0 = Vec2::X * 20.0;
        recharge_pulse(&mut world);
        assert_eq!(energy(&world, unit), 0);
        world.get_mut::<Position>(unit).unwrap().0 = Vec2::ZERO;
        world.despawn(root);
        recharge_pulse(&mut world);
        assert_eq!(energy(&world, unit), 0);
    }
    #[test]
    fn five_second_clock_scales_speed_and_freezes_paused_hidden_and_finished() {
        let mut world = world();
        let own = world.spawn(Faction { index: 0 }).id();
        tower(&mut world, own, 0.0);
        let unit = minion(&mut world, own, Vec2::ZERO, 0);
        world
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs(4));
        recharge_energy(&mut world);
        assert_eq!(energy(&world, unit), 0);
        for mode in 0..3 {
            {
                let mut state = world.resource_mut::<MatchState>();
                state.paused = mode == 0;
                state.browser_hidden = mode == 1;
                state.winner = (mode == 2).then_some(0);
            }
            recharge_energy(&mut world);
            assert_eq!(world.resource::<EnergyClock>().elapsed, 4.0);
        }
        *world.resource_mut::<MatchState>() = MatchState::default();
        world.resource_mut::<MatchState>().speed = 2.0;
        world
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs(3));
        recharge_energy(&mut world);
        assert_eq!(world.resource::<EnergyClock>().pulses, 2);
        assert_eq!(world.resource::<EnergyClock>().elapsed, 0.0);
        assert_eq!(energy(&world, unit), 2);
    }
}
