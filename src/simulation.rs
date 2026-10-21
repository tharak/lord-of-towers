//! Deterministic, renderer-independent match systems.
// ECS system parameters describe independent access; splitting them obscures scheduling.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use std::collections::HashMap;

use bevy::prelude::*;

use crate::{influence::InfluenceField, model::*};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationConfig>()
            .init_resource::<MatchState>()
            .init_resource::<FactionEntities>()
            .init_resource::<SpatialIndex>()
            .init_resource::<PendingDamage>()
            .init_resource::<SpawnSequence>()
            .insert_resource(RandomStream(42))
            .insert_resource(Time::<Fixed>::from_hz(30.0))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::Spawn,
                    SimSet::Index,
                    SimSet::Target,
                    SimSet::Movement,
                    SimSet::Combat,
                    SimSet::Capture,
                    SimSet::Gathering,
                    SimSet::PaidSpawn,
                    SimSet::Energy,
                    SimSet::Influence,
                    SimSet::Victory,
                )
                    .chain()
                    .run_if(simulation_running),
            )
            .add_systems(Startup, setup_match)
            .add_systems(PreUpdate, restart_match)
            .add_systems(FixedUpdate, advance_clock.in_set(SimSet::Spawn))
            .add_systems(FixedUpdate, rebuild_index.in_set(SimSet::Index))
            .add_systems(FixedUpdate, move_minions.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                (rebuild_index, collect_damage)
                    .chain()
                    .in_set(SimSet::Combat),
            )
            .add_systems(FixedUpdate, resolve_damage.in_set(SimSet::Capture))
            .add_systems(FixedUpdate, spawn_minions.in_set(SimSet::PaidSpawn))
            .add_systems(FixedUpdate, check_victory.in_set(SimSet::Victory));
    }
}

#[derive(Resource, Default)]
struct SpawnSequence(u64);

#[derive(Resource)]
struct RandomStream(u64);
impl RandomStream {
    fn unit(&mut self) -> f32 {
        // SplitMix64 also handles seed zero. This sequence is platform-independent.
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut n = self.0;
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((n ^ (n >> 31)) >> 40) as f32 / 16_777_216.0
    }
    fn direction(&mut self) -> Vec2 {
        Vec2::from_angle(self.unit() * std::f32::consts::TAU)
    }
}

#[derive(Clone, Copy)]
struct Neighbor {
    entity: Entity,
    position: Vec2,
    faction: Entity,
    tower: bool,
    stable_id: u64,
}

#[derive(Resource, Default)]
struct SpatialIndex(HashMap<(i32, i32), Vec<Neighbor>>);
impl SpatialIndex {
    const CELL: f32 = 8.0;
    fn cell(position: Vec2) -> (i32, i32) {
        (
            (position.x / Self::CELL).floor() as i32,
            (position.y / Self::CELL).floor() as i32,
        )
    }
    fn nearby(&self, position: Vec2, radius: f32) -> impl Iterator<Item = &Neighbor> {
        let (min_x, min_y) = Self::cell(position - Vec2::splat(radius));
        let (max_x, max_y) = Self::cell(position + Vec2::splat(radius));
        (min_x..=max_x)
            .flat_map(move |x| {
                (min_y..=max_y).flat_map(move |y| self.0.get(&(x, y)).into_iter().flatten())
            })
            .filter(move |entry| position.distance_squared(entry.position) <= radius * radius)
    }
}

#[derive(Resource, Default)]
struct PendingDamage(HashMap<Entity, [f32; 5]>);

fn setup_match(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut factions: ResMut<FactionEntities>,
    mut random: ResMut<RandomStream>,
    mut sequence: ResMut<SpawnSequence>,
) {
    sequence.0 = 5;
    create_match(
        &mut commands,
        &config,
        &mut factions,
        &mut random,
        &mut sequence,
    );
}

fn create_match(
    commands: &mut Commands,
    config: &SimulationConfig,
    factions: &mut FactionEntities,
    random: &mut RandomStream,
    sequence: &mut SpawnSequence,
) {
    random.0 = config.seed;
    factions.0.clear();
    for (index, name) in FACTION_NAMES.iter().enumerate() {
        let faction = commands
            .spawn((
                Name::new(*name),
                EntityKind::Faction,
                MatchEntity,
                Faction { index },
            ))
            .id();
        factions.0.push(faction);
        let angle = index as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
        let position = Vec2::from_angle(angle) * 32.0_f32.min(config.half_extent * 0.75);
        let tower = commands
            .spawn((
                Name::new(format!("{name} tower")),
                EntityKind::Tower,
                MatchEntity,
                Tower { slot: index },
                ForwardingCursor::default(),
                SimulationId(index as u64),
                Allegiance(faction),
                Position(position),
                Health {
                    current: config.tower_health,
                    max: config.tower_health,
                },
                Spawner {
                    cap: config.population_cap,
                },
                RawElement(0),
                InfluenceEmitter {
                    radius: 15.0,
                    strength: 4.0,
                },
            ))
            .id();
        for _ in 0..config.starting_minions.min(config.population_cap) {
            spawn_minion(
                commands,
                config,
                random,
                sequence,
                tower,
                position,
                Allegiance(faction),
            );
        }
    }
    crate::gathering::spawn_deposits(commands, config);
}

fn restart_match(
    mut commands: Commands,
    mut state: ResMut<MatchState>,
    config: Res<SimulationConfig>,
    entities: Query<Entity, With<MatchEntity>>,
    mut factions: ResMut<FactionEntities>,
    mut random: ResMut<RandomStream>,
    mut spatial: ResMut<SpatialIndex>,
    mut damage: ResMut<PendingDamage>,
    field: Option<ResMut<InfluenceField>>,
    mut sequence: ResMut<SpawnSequence>,
    energy_clock: Option<ResMut<EnergyClock>>,
) {
    if !state.restart_requested {
        return;
    }
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    let hidden = state.browser_hidden;
    *state = MatchState {
        browser_hidden: hidden,
        ..default()
    };
    spatial.0.clear();
    damage.0.clear();
    sequence.0 = 5;
    if let Some(mut clock) = energy_clock {
        *clock = EnergyClock::default();
    }
    if let Some(mut field) = field {
        field.clear();
    }
    create_match(
        &mut commands,
        &config,
        &mut factions,
        &mut random,
        &mut sequence,
    );
}

fn advance_clock(time: Res<Time<Fixed>>, mut state: ResMut<MatchState>) {
    state.elapsed += simulation_delta(&time, &state);
}

fn spawn_minion(
    commands: &mut Commands,
    config: &SimulationConfig,
    random: &mut RandomStream,
    sequence: &mut SpawnSequence,
    tower: Entity,
    position: Vec2,
    allegiance: Allegiance,
) {
    let direction = random.direction();
    let location = (position + direction * 2.8).clamp(
        Vec2::splat(-config.half_extent),
        Vec2::splat(config.half_extent),
    );
    let stable_id = sequence.0;
    sequence.0 += 1;
    commands.spawn((
        (
            SimulationId(stable_id),
            EnergyStore {
                current: config.initial_energy.min(config.energy_capacity),
                capacity: config.energy_capacity,
            },
            ForwardingCursor::default(),
            RawElement(0),
            Vision {
                radius: config.vision_radius,
            },
            GatheringState::Exploring,
        ),
        (
            Name::new("Minion"),
            EntityKind::Minion,
            MatchEntity,
            Minion,
            allegiance,
            OriginTower(tower),
            Position(location),
            Health {
                current: config.minion_health,
                max: config.minion_health,
            },
            Movement {
                speed: config.minion_speed,
                target: location + direction,
                retarget_in: 0.0,
            },
            Attack {
                damage: 10.0,
                range: 1.6,
                cooldown: 0.8,
                remaining: 0.0,
            },
            InfluenceEmitter {
                radius: 5.0,
                strength: 1.0,
            },
        ),
    ));
}

fn spawn_minions(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut random: ResMut<RandomStream>,
    mut towers: Query<(
        Entity,
        &Position,
        &Allegiance,
        &Spawner,
        &Tower,
        &mut RawElement,
    )>,
    mut sequence: ResMut<SpawnSequence>,
    minions: Query<&OriginTower, With<Minion>>,
    mut economy: ResMut<RawElementEconomy>,
) {
    let mut counts = HashMap::<Entity, usize>::new();
    for origin in &minions {
        *counts.entry(origin.0).or_default() += 1;
    }
    let mut order: Vec<_> = towers
        .iter()
        .map(|(entity, _, _, _, tower, _)| (tower.slot, entity))
        .collect();
    order.sort_by_key(|(slot, _)| *slot);
    // A zero configured cost must never create an unbounded free-spawning loop.
    let cost = config.minion_raw_cost.max(1);
    for (_, entity) in order {
        let Ok((tower, position, allegiance, spawner, _, mut stock)) = towers.get_mut(entity)
        else {
            continue;
        };
        let count = counts.entry(tower).or_default();
        while stock.0 >= cost && *count < spawner.cap {
            stock.0 -= cost;
            economy.spent += cost;
            *count += 1;
            spawn_minion(
                &mut commands,
                &config,
                &mut random,
                &mut sequence,
                tower,
                position.0,
                *allegiance,
            );
        }
    }
}

fn rebuild_index(
    mut spatial: ResMut<SpatialIndex>,
    units: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            Option<&Tower>,
            Option<&SimulationId>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
) {
    spatial.0.clear();
    for (entity, position, faction, tower, stable_id) in &units {
        spatial
            .0
            .entry(SpatialIndex::cell(position.0))
            .or_default()
            .push(Neighbor {
                entity,
                position: position.0,
                faction: faction.0,
                tower: tower.is_some(),
                stable_id: stable_id.map_or(entity.to_bits(), |id| id.0),
            });
    }
    for bucket in spatial.0.values_mut() {
        bucket.sort_by_key(|entry| entry.stable_id);
    }
}

fn move_minions(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    config: Res<SimulationConfig>,
    spatial: Res<SpatialIndex>,
    mut minions: Query<(Entity, &mut Position, &Movement), With<Minion>>,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, mut position, movement) in &mut minions {
        let offset = movement.target - position.0;
        let mut velocity =
            offset.normalize_or_zero() * movement.speed.min(offset.length() / dt.max(0.0001));
        for other in spatial
            .nearby(position.0, 1.1)
            .filter(|entry| entry.entity != entity && !entry.tower)
        {
            let away = position.0 - other.position;
            let distance = away.length();
            if distance > 0.001 {
                velocity += away / distance * (1.1 - distance) * 3.0;
            }
        }
        position.0 = (position.0 + velocity.clamp_length_max(movement.speed * 1.25) * dt).clamp(
            Vec2::splat(-config.half_extent),
            Vec2::splat(config.half_extent),
        );
    }
}

fn collect_damage(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    spatial: Res<SpatialIndex>,
    factions: Query<&Faction>,
    mut pending: ResMut<PendingDamage>,
    mut attackers: Query<
        (
            &Position,
            &Allegiance,
            &mut Attack,
            &mut EnergyStore,
            &Vision,
        ),
        With<Minion>,
    >,
    config: Res<SimulationConfig>,
) {
    pending.0.clear();
    let dt = simulation_delta(&time, &state);
    for (position, faction, mut attack, mut energy, vision) in &mut attackers {
        attack.remaining = (attack.remaining - dt).max(0.0);
        if attack.remaining > 0.0 || energy.current < config.attack_energy_cost {
            continue;
        }
        let Ok(faction) = factions
            .get(faction.0)
            .map(|record| (faction.0, record.index))
        else {
            continue;
        };
        let target = spatial
            .nearby(position.0, attack.range + 2.0)
            .filter(|entry| {
                entry.faction != faction.0
                    && position.0.distance_squared(entry.position) <= vision.radius.max(0.0).powi(2)
                    && position.0.distance(entry.position)
                        <= attack.range + if entry.tower { 2.0 } else { 0.0 }
            })
            .min_by(|a, b| {
                a.tower
                    .cmp(&b.tower)
                    .then_with(|| {
                        position
                            .0
                            .distance_squared(a.position)
                            .total_cmp(&position.0.distance_squared(b.position))
                    })
                    .then(a.stable_id.cmp(&b.stable_id))
            });
        if let Some(target) = target {
            pending.0.entry(target.entity).or_default()[faction.1] += attack.damage;
            attack.remaining = attack.cooldown;
            energy.current -= config.attack_energy_cost;
        }
    }
}

fn resolve_damage(
    mut commands: Commands,
    mut pending: ResMut<PendingDamage>,
    factions: Res<FactionEntities>,
    mut units: Query<
        (
            &mut Health,
            &mut Allegiance,
            Option<&mut Spawner>,
            Option<&mut ForwardingCursor>,
            &Position,
            Option<&RawElement>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
    ids: Query<&SimulationId>,
) {
    let mut hits: Vec<_> = pending.0.drain().collect();
    hits.sort_by_key(|(entity, _)| ids.get(*entity).map_or(entity.to_bits(), |id| id.0));
    for (entity, contributions) in hits {
        let Ok((mut health, mut allegiance, spawner, cursor, position, cargo)) =
            units.get_mut(entity)
        else {
            continue;
        };
        health.current -= contributions.iter().sum::<f32>();
        if health.current > 0.0 {
            continue;
        }
        if spawner.is_some() {
            let mut winner = 0;
            for index in 1..5 {
                if contributions[index] > contributions[winner] {
                    winner = index;
                }
            }
            if let Some(faction) = factions.0.get(winner) {
                allegiance.0 = *faction;
                health.current = health.max;
                if let Some(mut cursor) = cursor {
                    *cursor = ForwardingCursor::default();
                }
            }
        } else {
            if let Some(cargo) = cargo
                && cargo.0 > 0
            {
                crate::gathering::drop_cargo(&mut commands, position.0, cargo.0);
            }
            commands.entity(entity).despawn();
        }
    }
}

fn check_victory(
    towers: Query<&Allegiance, With<Tower>>,
    factions: Query<&Faction>,
    mut state: ResMut<MatchState>,
) {
    let mut ownership = towers.iter();
    let Some(first) = ownership.next() else {
        return;
    };
    if ownership.all(|owner| owner == first)
        && let Ok(faction) = factions.get(first.0)
    {
        state.winner = Some(faction.index);
        state.paused = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins((SimulationPlugin, crate::gathering::GatheringPlugin));
        app.update();
        app
    }
    fn step(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(std::time::Duration::from_secs_f64(1.0 / 30.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }
    fn count<T: Component>(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<T>>()
            .iter(app.world())
            .count()
    }
    fn total(app: &mut App) -> u32 {
        app.world_mut()
            .query::<&RawElement>()
            .iter(app.world())
            .map(|raw| raw.0)
            .sum::<u32>()
            + app.world().resource::<RawElementEconomy>().spent
    }
    fn tower(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<Tower>>()
            .iter(app.world())
            .next()
            .unwrap()
    }

    #[test]
    fn startup_armies_have_stable_ids_and_complete_gathering_components() {
        let mut app = app();
        assert_eq!(count::<Minion>(&mut app), 15);
        assert_eq!(count::<RawElementDeposit>(&mut app), 30);
        assert_eq!(app.world().resource::<SpawnSequence>().0, 20);
        let mut ids = Vec::new();
        for (id, energy, raw, vision, state, _) in app
            .world_mut()
            .query_filtered::<(
                &SimulationId,
                &EnergyStore,
                &RawElement,
                &Vision,
                &GatheringState,
                &ForwardingCursor,
            ), With<Minion>>()
            .iter(app.world())
        {
            ids.push(id.0);
            assert_eq!((energy.current, energy.capacity, raw.0), (5, 5, 0));
            assert_eq!(vision.radius, 15.0);
            assert!(matches!(state, GatheringState::Exploring));
        }
        ids.sort_unstable();
        assert_eq!(ids, (5..20).collect::<Vec<_>>());
        assert_eq!(
            total(&mut app),
            app.world().resource::<RawElementEconomy>().initial
        );
    }

    #[test]
    fn paid_spawning_spends_all_affordable_stock_and_respects_origin_cap() {
        let mut app = app();
        let tower = tower(&mut app);
        app.world_mut().get_mut::<RawElement>(tower).unwrap().0 = 17;
        app.world_mut().get_mut::<Spawner>(tower).unwrap().cap = 5;
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_minions);
        schedule.run(app.world_mut());
        assert_eq!(count::<Minion>(&mut app), 17);
        assert_eq!(app.world().get::<RawElement>(tower).unwrap().0, 7);
        assert_eq!(app.world().resource::<RawElementEconomy>().spent, 10);
        schedule.run(app.world_mut());
        assert_eq!(count::<Minion>(&mut app), 17);
        assert_eq!(app.world().get::<RawElement>(tower).unwrap().0, 7);
        app.world_mut().get_mut::<Spawner>(tower).unwrap().cap = 100;
        schedule.run(app.world_mut());
        assert_eq!(count::<Minion>(&mut app), 18);
        assert_eq!(app.world().get::<RawElement>(tower).unwrap().0, 2);
    }

    #[test]
    fn capture_retains_stock_and_old_army_and_resets_forwarding() {
        let mut app = app();
        let tower = tower(&mut app);
        let factions = app.world().resource::<FactionEntities>().0.clone();
        let old_owner = app.world().get::<Allegiance>(tower).unwrap().0;
        let army: Vec<_> = app
            .world_mut()
            .query::<(Entity, &OriginTower)>()
            .iter(app.world())
            .filter(|(_, origin)| origin.0 == tower)
            .map(|(entity, _)| entity)
            .collect();
        app.world_mut().get_mut::<RawElement>(tower).unwrap().0 = 4;
        app.world_mut()
            .get_mut::<ForwardingCursor>(tower)
            .unwrap()
            .last_recipient = Some(123);
        app.world_mut()
            .resource_mut::<PendingDamage>()
            .0
            .insert(tower, [0.0, 200.0, 200.0, 0.0, 0.0]);
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_damage);
        schedule.run(app.world_mut());
        assert_eq!(app.world().get::<Allegiance>(tower).unwrap().0, factions[1]);
        assert_eq!(app.world().get::<RawElement>(tower).unwrap().0, 4);
        assert_eq!(
            app.world()
                .get::<ForwardingCursor>(tower)
                .unwrap()
                .last_recipient,
            None
        );
        for entity in army {
            assert_eq!(app.world().get::<Allegiance>(entity).unwrap().0, old_owner);
        }
        assert_eq!(count::<Minion>(&mut app), 15);
    }

    #[test]
    fn death_drops_cargo_without_losing_resource() {
        let mut app = app();
        let minion = app
            .world_mut()
            .query_filtered::<Entity, With<Minion>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().get_mut::<RawElement>(minion).unwrap().0 = 1;
        let before = total(&mut app);
        let deposits = count::<RawElementDeposit>(&mut app);
        app.world_mut()
            .resource_mut::<PendingDamage>()
            .0
            .insert(minion, [100.0, 0.0, 0.0, 0.0, 0.0]);
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_damage);
        schedule.run(app.world_mut());
        assert!(app.world().get_entity(minion).is_err());
        assert_eq!(count::<RawElementDeposit>(&mut app), deposits + 1);
        assert_eq!(total(&mut app), before);
    }

    #[test]
    fn combat_prioritizes_eligible_minions_then_distance_and_stable_id() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        let center = Vec2::splat(1000.0);
        let attacker = app
            .world_mut()
            .spawn((
                Minion,
                Position(center),
                Allegiance(factions[0]),
                Vision { radius: 3.0 },
                EnergyStore {
                    current: 5,
                    capacity: 5,
                },
                Attack {
                    damage: 10.0,
                    range: 1.6,
                    cooldown: 0.8,
                    remaining: 0.0,
                },
            ))
            .id();
        let tower = app
            .world_mut()
            .spawn((
                Tower { slot: 99 },
                Position(center + Vec2::X * 0.25),
                Allegiance(factions[1]),
                SimulationId(100),
            ))
            .id();
        let farther = app
            .world_mut()
            .spawn((
                Minion,
                Position(center + Vec2::X * 1.5),
                Allegiance(factions[1]),
                SimulationId(101),
            ))
            .id();
        // Spawn the larger stable ID first so entity order cannot break the tie.
        let tied = app
            .world_mut()
            .spawn((
                Minion,
                Position(center - Vec2::X),
                Allegiance(factions[1]),
                SimulationId(103),
            ))
            .id();
        let preferred = app
            .world_mut()
            .spawn((
                Minion,
                Position(center + Vec2::X),
                Allegiance(factions[1]),
                SimulationId(102),
            ))
            .id();
        let mut combat = Schedule::default();
        combat.add_systems((rebuild_index, collect_damage).chain());
        combat.run(app.world_mut());
        let damage = &app.world().resource::<PendingDamage>().0;
        assert_eq!(damage.get(&preferred).unwrap()[0], 10.0);
        for other in [tower, farther, tied] {
            assert!(!damage.contains_key(&other));
        }
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 4);
        assert_eq!(app.world().get::<Attack>(attacker).unwrap().remaining, 0.8);

        // Cooldown prevents another strike even with eligible targets present.
        combat.run(app.world_mut());
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 4);

        // A visible minion outside attack range must not prevent tower fallback.
        for minion in [preferred, farther, tied] {
            app.world_mut().get_mut::<Position>(minion).unwrap().0 = center + Vec2::X * 2.0;
        }
        app.world_mut()
            .get_mut::<Attack>(attacker)
            .unwrap()
            .remaining = 0.0;
        combat.run(app.world_mut());
        assert_eq!(
            app.world()
                .resource::<PendingDamage>()
                .0
                .get(&tower)
                .unwrap()[0],
            10.0
        );
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 3);

        // Without an eligible target, neither energy nor cooldown is spent.
        app.world_mut().get_mut::<Position>(tower).unwrap().0 = center + Vec2::X * 4.0;
        app.world_mut()
            .get_mut::<Attack>(attacker)
            .unwrap()
            .remaining = 0.0;
        combat.run(app.world_mut());
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 3);
        assert_eq!(app.world().get::<Attack>(attacker).unwrap().remaining, 0.0);
    }

    #[test]
    fn combat_requires_energy_even_with_an_eligible_target() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        let center = Vec2::splat(1000.0);
        let attacker = app
            .world_mut()
            .spawn((
                Minion,
                Position(center),
                Allegiance(factions[0]),
                Vision { radius: 3.0 },
                EnergyStore {
                    current: 0,
                    capacity: 5,
                },
                Attack {
                    damage: 10.0,
                    range: 1.6,
                    cooldown: 0.8,
                    remaining: 0.0,
                },
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((Minion, Position(center + Vec2::X), Allegiance(factions[1])))
            .id();
        let mut combat = Schedule::default();
        combat.add_systems((rebuild_index, collect_damage).chain());
        combat.run(app.world_mut());
        assert!(
            !app.world()
                .resource::<PendingDamage>()
                .0
                .contains_key(&target)
        );
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 0);
        assert_eq!(app.world().get::<Attack>(attacker).unwrap().remaining, 0.0);
    }

    #[test]
    fn combat_uses_actual_inclusive_vision_and_keeps_gathering_movement() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        let attacker = app
            .world_mut()
            .spawn((
                Minion,
                Position(Vec2::ZERO),
                Allegiance(factions[0]),
                Vision { radius: 1.0 },
                EnergyStore {
                    current: 5,
                    capacity: 5,
                },
                Attack {
                    damage: 10.0,
                    range: 1.6,
                    cooldown: 0.8,
                    remaining: 0.0,
                },
                Movement {
                    speed: 4.0,
                    target: Vec2::Y * 10.0,
                    retarget_in: 99.0,
                },
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((Minion, Position(Vec2::X * 1.01), Allegiance(factions[1])))
            .id();
        let mut combat = Schedule::default();
        combat.add_systems((rebuild_index, collect_damage).chain());
        combat.run(app.world_mut());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 5);
        app.world_mut().get_mut::<Position>(target).unwrap().0 = Vec2::X;
        combat.run(app.world_mut());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 4);
        assert!(
            app.world()
                .resource::<PendingDamage>()
                .0
                .contains_key(&target)
        );
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 30.0));
        let mut movement = Schedule::default();
        movement.add_systems(move_minions);
        movement.run(app.world_mut());
        assert!(app.world().get::<Position>(attacker).unwrap().0.y > 0.0);
        assert_eq!(
            app.world().get::<Movement>(attacker).unwrap().target,
            Vec2::Y * 10.0
        );
    }

    #[test]
    fn pause_hidden_and_victory_freeze_simulation() {
        let mut app = app();
        let initial = total(&mut app);
        for mode in 0..3 {
            {
                let mut state = app.world_mut().resource_mut::<MatchState>();
                state.paused = mode == 0;
                state.browser_hidden = mode == 1;
                state.winner = (mode == 2).then_some(0);
            }
            step(&mut app, 90);
            assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
            assert_eq!(count::<Minion>(&mut app), 15);
            assert_eq!(total(&mut app), initial);
        }
    }

    #[test]
    fn restart_resets_resources_layout_armies_links_and_clock() {
        fn layout(app: &mut App) -> Vec<(u64, Vec2, u32)> {
            let mut result: Vec<_> = app
                .world_mut()
                .query_filtered::<(&SimulationId, &Position, &RawElement), With<RawElementDeposit>>(
                )
                .iter(app.world())
                .map(|(id, position, raw)| (id.0, position.0, raw.0))
                .collect();
            result.sort_by_key(|entry| entry.0);
            result
        }
        let mut app = app();
        app.add_plugins(crate::energy::EnergyPlugin);
        let original = layout(&mut app);
        let old = app.world().resource::<FactionEntities>().0.clone();
        step(&mut app, 200);
        app.world_mut()
            .resource_mut::<MatchState>()
            .restart_requested = true;
        app.world_mut().run_schedule(PreUpdate);
        assert_eq!(layout(&mut app), original);
        assert_eq!(count::<Minion>(&mut app), 15);
        assert_eq!(count::<Tower>(&mut app), 5);
        assert_eq!(count::<Trail>(&mut app), 0);
        assert_eq!(count::<Link>(&mut app), 0);
        assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
        assert_eq!(app.world().resource::<EnergyClock>().pulses, 0);
        assert_eq!(app.world().resource::<EnergyClock>().elapsed, 0.0);
        assert_eq!(app.world().resource::<RawElementEconomy>().spent, 0);
        assert_eq!(
            total(&mut app),
            app.world().resource::<RawElementEconomy>().initial
        );
        for entity in old {
            assert!(app.world().get_entity(entity).is_err());
        }
        for raw in app
            .world_mut()
            .query_filtered::<&RawElement, Or<(With<Minion>, With<Tower>)>>()
            .iter(app.world())
        {
            assert_eq!(raw.0, 0);
        }
    }

    #[test]
    #[ignore = "extended run invoked by scheduled/manual GitHub Actions"]
    fn extended_simulation_remains_bounded_and_conserves_raw_element() {
        let mut app = app();
        app.add_plugins(crate::energy::EnergyPlugin);
        let initial = app.world().resource::<RawElementEconomy>().initial;
        for _ in 0..180 {
            step(&mut app, 100);
            let minions = count::<Minion>(&mut app);
            assert!(minions <= 500);
            assert_eq!(count::<Tower>(&mut app), 5);
            assert!(count::<RawElementDeposit>(&mut app) as u32 <= initial);
            assert!(count::<Trail>(&mut app) <= 500 * 100);
            assert!(count::<Link>(&mut app) <= minions);
            assert_eq!(total(&mut app), initial);
            for energy in app.world_mut().query::<&EnergyStore>().iter(app.world()) {
                assert!(energy.current <= energy.capacity);
            }
            for (position, health) in app
                .world_mut()
                .query::<(&Position, &Health)>()
                .iter(app.world())
            {
                assert!(position.0.is_finite());
                assert!(position.0.abs().max_element() <= 50.0);
                assert!(health.current > 0.0 && health.current <= health.max);
            }
        }
    }
    #[test]
    fn trails_are_not_combat_or_separation_targets() {
        let mut app = app();
        let owner = app.world().resource::<FactionEntities>().0[0];
        let trail = app
            .world_mut()
            .spawn((
                Position(Vec2::ZERO),
                Allegiance(owner),
                Trail {
                    trip: 0,
                    sequence: 0,
                    remaining: 30.0,
                },
            ))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(rebuild_index);
        schedule.run(app.world_mut());
        assert!(
            !app.world()
                .resource::<SpatialIndex>()
                .nearby(Vec2::ZERO, 100.0)
                .any(|entry| entry.entity == trail)
        );
    }
}
