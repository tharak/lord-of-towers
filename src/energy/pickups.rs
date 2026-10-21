//! Permanent neutral pickups; their layout never consumes the unit AI random stream.
#![allow(clippy::type_complexity)]
use crate::model::*;
use bevy::prelude::*;

pub fn spawn_pickups(commands: &mut Commands, config: &SimulationConfig) {
    let mut seed = config.seed ^ 0x7265_6368_6172_6765;
    let mut random = || {
        seed = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut bits = seed;
        bits = (bits ^ (bits >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        bits = (bits ^ (bits >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((bits ^ (bits >> 31)) >> 40) as f32 / 16_777_216.0
    };
    let margin = (config.half_extent - 2.0).max(0.0);
    let towers: Vec<_> = (0..5)
        .map(|slot| {
            Vec2::from_angle(
                slot as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2,
            ) * 32.0_f32.min(config.half_extent * 0.75)
        })
        .collect();
    let mut positions: Vec<Vec2> = Vec::new();
    for index in 0..config.energy_pickup_count {
        // Bounded attempts make even impossible custom densities safe to initialize.
        let position = (0..512).find_map(|_| {
            let candidate = Vec2::new(random() * 2.0 - 1.0, random() * 2.0 - 1.0) * margin;
            (towers
                .iter()
                .all(|point| point.distance_squared(candidate) >= 25.0)
                && positions
                    .iter()
                    .all(|point| point.distance_squared(candidate) >= 9.0))
            .then_some(candidate)
        });
        let Some(position) = position else {
            continue;
        };
        positions.push(position);
        commands.spawn((
            Name::new(format!("Energy pickup {}", index + 1)),
            EntityKind::EnergyPickup,
            MatchEntity,
            SimulationId((1_u64 << 63) + index as u64),
            Position(position),
            EnergyPickup {
                available: true,
                recharge_remaining: 0.0,
            },
        ));
    }
}

pub(super) fn recharge_pickups(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    mut pickups: Query<&mut EnergyPickup>,
) {
    let dt = simulation_delta(&time, &state);
    for mut pickup in &mut pickups {
        if pickup.available {
            continue;
        }
        pickup.recharge_remaining = (pickup.recharge_remaining - dt).max(0.0);
        if pickup.recharge_remaining <= f32::EPSILON {
            pickup.available = true;
            pickup.recharge_remaining = 0.0;
        }
    }
}

pub(super) fn collect_pickups(
    config: Res<SimulationConfig>,
    mut pickups: Query<(Entity, &SimulationId, &Position, &mut EnergyPickup)>,
    mut minions: Query<(Entity, &SimulationId, &Position, &mut EnergyStore), With<Minion>>,
) {
    let mut order: Vec<_> = pickups
        .iter()
        .map(|(entity, id, _, _)| (id.0, entity))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (_, entity) in order {
        let Ok((_, _, position, mut pickup)) = pickups.get_mut(entity) else {
            continue;
        };
        if !pickup.available {
            continue;
        }
        let collector = minions
            .iter()
            .filter(|(_, _, point, energy)| {
                energy.current < energy.capacity
                    && point.0.distance_squared(position.0) <= config.pickup_radius.powi(2)
            })
            .min_by(|(_, id_a, a, _), (_, id_b, b, _)| {
                a.0.distance_squared(position.0)
                    .total_cmp(&b.0.distance_squared(position.0))
                    .then(id_a.0.cmp(&id_b.0))
            })
            .map(|(entity, _, _, _)| entity);
        if let Some(collector) = collector {
            let Ok((_, _, _, mut energy)) = minions.get_mut(collector) else {
                continue;
            };
            energy.current += 1;
            pickup.available = false;
            pickup.recharge_remaining = config.pickup_recharge_seconds.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<SimulationConfig>()
            .init_resource::<MatchState>()
            .insert_resource(Time::<Fixed>::from_hz(30.0));
        app
    }
    fn collect(app: &mut App) {
        let mut schedule = Schedule::default();
        schedule.add_systems(collect_pickups);
        schedule.run(app.world_mut());
    }
    fn pickup(app: &mut App, id: u64, position: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                SimulationId((1 << 63) + id),
                Position(position),
                EnergyPickup {
                    available: true,
                    recharge_remaining: 0.0,
                },
            ))
            .id()
    }
    fn minion(app: &mut App, id: u64, position: Vec2, current: u8) -> Entity {
        app.world_mut()
            .spawn((
                Minion,
                SimulationId(id),
                Position(position),
                EnergyStore {
                    current,
                    capacity: 5,
                },
            ))
            .id()
    }

    #[test]
    fn collection_breaks_distance_ties_by_stable_id_and_never_overfills() {
        let mut app = app();
        let first = pickup(&mut app, 0, Vec2::ZERO);
        let second = pickup(&mut app, 1, Vec2::ZERO);
        let larger_id = minion(&mut app, 9, Vec2::X * 0.5, 4);
        let smaller_id = minion(&mut app, 7, -Vec2::X * 0.5, 4);
        collect(&mut app);
        for entity in [larger_id, smaller_id] {
            assert_eq!(app.world().get::<EnergyStore>(entity).unwrap().current, 5);
        }
        for entity in [first, second] {
            assert!(!app.world().get::<EnergyPickup>(entity).unwrap().available);
        }
        // With a single pickup the smaller stable ID wins, independent of entity creation.
        app.world_mut()
            .get_mut::<EnergyStore>(larger_id)
            .unwrap()
            .current = 4;
        app.world_mut()
            .get_mut::<EnergyStore>(smaller_id)
            .unwrap()
            .current = 4;
        app.world_mut()
            .get_mut::<EnergyPickup>(first)
            .unwrap()
            .available = true;
        collect(&mut app);
        assert_eq!(
            app.world().get::<EnergyStore>(smaller_id).unwrap().current,
            5
        );
        assert_eq!(
            app.world().get::<EnergyStore>(larger_id).unwrap().current,
            4
        );
    }

    #[test]
    fn full_and_out_of_range_units_do_not_consume_and_recharge_takes_five_seconds() {
        let mut app = app();
        let pickup = pickup(&mut app, 0, Vec2::ZERO);
        let full = minion(&mut app, 0, Vec2::ZERO, 5);
        minion(&mut app, 1, Vec2::X * 1.01, 0);
        collect(&mut app);
        assert!(app.world().get::<EnergyPickup>(pickup).unwrap().available);
        app.world_mut()
            .get_mut::<EnergyStore>(full)
            .unwrap()
            .current = 0;
        collect(&mut app);
        assert_eq!(app.world().get::<EnergyStore>(full).unwrap().current, 1);
        let mut recharge = Schedule::default();
        recharge.add_systems(recharge_pickups);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs(4));
        recharge.run(app.world_mut());
        assert!(!app.world().get::<EnergyPickup>(pickup).unwrap().available);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs(1));
        recharge.run(app.world_mut());
        assert!(app.world().get::<EnergyPickup>(pickup).unwrap().available);
    }
}
