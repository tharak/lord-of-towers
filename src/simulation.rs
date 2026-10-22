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
            .init_resource::<PendingTowerAlerts>()
            .init_resource::<SpawnSequence>()
            .insert_resource(RandomStream(42))
            .insert_resource(Time::<Fixed>::from_hz(30.0))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::Spawn,
                    SimSet::Index,
                    SimSet::Target,
                    SimSet::CombatTarget,
                    SimSet::TrailInput,
                    SimSet::Movement,
                    SimSet::Combat,
                    SimSet::Capture,
                    SimSet::Gathering,
                    SimSet::TowerGrowth,
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
            .add_systems(
                FixedUpdate,
                select_combat_targets.in_set(SimSet::CombatTarget),
            )
            .add_systems(FixedUpdate, move_minions.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                (
                    rebuild_index,
                    clear_pending_damage,
                    advance_projectiles,
                    collect_damage,
                    process_tower_alerts,
                )
                    .chain()
                    .in_set(SimSet::Combat),
            )
            .add_systems(FixedUpdate, resolve_damage.in_set(SimSet::Capture))
            .add_systems(FixedUpdate, grow_tower_radius.in_set(SimSet::TowerGrowth))
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
                SimulationId(index as u64),
                Allegiance(faction),
                Position(position),
                Health {
                    current: config.tower_health,
                    max: config.tower_health,
                },
                RawElement(0),
                InfluenceEmitter {
                    radius: config.tower_influence_radius,
                    strength: 4.0,
                },
            ))
            .id();
        if index == 0 {
            commands.entity(tower).insert(PlayerControlled);
        }
        for _ in 0..config.starting_minions {
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
    mut alerts: ResMut<PendingTowerAlerts>,
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
    alerts.0.clear();
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
            RawElement(0),
            Vision {
                radius: config.vision_radius,
            },
            GatheringState::Exploring,
            ShootingPause::default(),
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
                range: 6.0,
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

/// Raw element remains stored: each unit expands the tower's circular area.
fn grow_tower_radius(
    config: Res<SimulationConfig>,
    mut towers: Query<(&RawElement, &mut InfluenceEmitter), With<Tower>>,
) {
    for (stock, mut emitter) in &mut towers {
        emitter.radius =
            config.tower_influence_radius + stock.0 as f32 * config.tower_radius_per_raw;
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
            Option<&Health>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
) {
    spatial.0.clear();
    for (entity, position, faction, tower, stable_id, health) in &units {
        if health.is_some_and(|health| health.current <= 0.0) {
            continue;
        }
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
    mut minions: Query<
        (Entity, &mut Position, &Movement, Option<&mut ShootingPause>),
        With<Minion>,
    >,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, mut position, movement, pause) in &mut minions {
        if let Some(mut pause) = pause
            && pause.0 > 0.0
        {
            pause.0 = (pause.0 - dt).max(0.0);
            continue;
        }
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

fn clear_pending_damage(mut pending: ResMut<PendingDamage>) {
    pending.0.clear();
}

fn advance_projectiles(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    factions: Query<&Faction>,
    targets: Query<(&Position, &Allegiance, &Health), Without<EnergyProjectile>>,
    mut projectiles: Query<(Entity, &mut Position, &Allegiance, &mut EnergyProjectile)>,
    mut pending: ResMut<PendingDamage>,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, mut position, owner, mut projectile) in &mut projectiles {
        let Ok((target_position, target_owner, health)) = targets.get(projectile.target) else {
            commands.entity(entity).despawn();
            continue;
        };
        if target_owner == owner || health.current <= 0.0 || projectile.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let offset = target_position.0 - position.0;
        let travel = projectile.speed.max(0.0) * dt.min(projectile.remaining);
        if offset.length() <= travel {
            if let Ok(faction) = factions.get(owner.0) {
                pending.0.entry(projectile.target).or_default()[faction.index] += projectile.damage;
            }
            commands.entity(entity).despawn();
        } else {
            position.0 += offset.normalize_or_zero() * travel;
            projectile.remaining -= dt;
            if projectile.remaining <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Combat overrides gathering only for a visible opponent and available attack energy.
fn select_combat_targets(
    mut commands: Commands,
    spatial: Res<SpatialIndex>,
    config: Res<SimulationConfig>,
    towers: Query<(&Health, &Allegiance), With<Tower>>,
    mut minions: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            &Vision,
            &EnergyStore,
            &Attack,
            Option<&RawElement>,
            Option<&TowerDefenseTarget>,
            Option<&mut Movement>,
        ),
        With<Minion>,
    >,
) {
    for (entity, position, owner, vision, energy, attack, cargo, defense, movement) in &mut minions
    {
        let visible = |target: Entity| {
            spatial
                .nearby(position.0, vision.radius.max(0.0))
                .find(|entry| entry.entity == target && entry.faction != owner.0)
        };
        let defense_target = defense.and_then(|defense| {
            towers
                .get(defense.tower)
                .ok()
                .filter(|(health, tower_owner)| health.current > 0.0 && *tower_owner == owner)?;
            visible(defense.attacker)
        });
        if defense.is_some() && defense_target.is_none() {
            commands.entity(entity).remove::<TowerDefenseTarget>();
        }
        if energy.current < config.attack_energy_cost {
            commands.entity(entity).remove::<CombatTarget>();
            continue;
        }
        let carrying = cargo.is_some_and(|cargo| cargo.0 > 0);
        let target = defense_target.or_else(|| {
            spatial
                .nearby(position.0, vision.radius.max(0.0))
                .filter(|entry| entry.faction != owner.0)
                .filter(|entry| {
                    !carrying
                        || position.0.distance(entry.position)
                            <= attack.range + if entry.tower { 2.0 } else { 0.0 }
                })
                .min_by(|a, b| {
                    position
                        .0
                        .distance_squared(a.position)
                        .total_cmp(&position.0.distance_squared(b.position))
                        .then(a.stable_id.cmp(&b.stable_id))
                })
        });
        if let Some(target) = target {
            commands.entity(entity).insert(CombatTarget(target.entity));
            if (!carrying || defense_target.is_some())
                && let Some(mut movement) = movement
            {
                let offset = target.position - position.0;
                let range = attack.range + if target.tower { 2.0 } else { 0.0 };
                movement.target =
                    target.position - offset.normalize_or_zero() * range.min(offset.length());
                movement.retarget_in = 0.0;
            }
        } else {
            commands.entity(entity).remove::<CombatTarget>();
        }
    }
}

fn process_tower_alerts(
    mut commands: Commands,
    mut alerts: ResMut<PendingTowerAlerts>,
    towers: Query<(&Position, &Allegiance, &InfluenceEmitter, &Health), With<Tower>>,
    targets: Query<
        (&Position, &Allegiance, &Health, Option<&SimulationId>),
        Or<(With<Minion>, With<Tower>)>,
    >,
    minions: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            &Vision,
            Option<&TowerDefenseTarget>,
        ),
        With<Minion>,
    >,
) {
    let alerts = std::mem::take(&mut alerts.0);
    for (entity, position, owner, vision, current) in &minions {
        let valid = |defense: TowerDefenseTarget| {
            let (_, tower_owner, _, tower_health) = towers.get(defense.tower).ok()?;
            let (attacker_position, attacker_owner, health, id) =
                targets.get(defense.attacker).ok()?;
            let distance = position.0.distance_squared(attacker_position.0);
            (tower_owner == owner
                && tower_health.current > 0.0
                && attacker_owner != owner
                && health.current > 0.0
                && distance <= vision.radius.max(0.0).powi(2))
            .then_some((
                defense,
                distance,
                id.map_or(defense.attacker.to_bits(), |id| id.0),
            ))
        };
        let best = current
            .copied()
            .and_then(valid)
            .into_iter()
            .chain(alerts.iter().filter_map(|alert| {
                let (tower_position, tower_owner, influence, _) = towers.get(alert.tower).ok()?;
                if tower_owner != owner
                    || position.0.distance_squared(tower_position.0)
                        > influence.radius.max(0.0).powi(2)
                {
                    return None;
                }
                valid(TowerDefenseTarget {
                    tower: alert.tower,
                    attacker: alert.attacker,
                })
            }))
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.cmp(&b.2)));
        if let Some((defense, _, _)) = best {
            commands.entity(entity).insert(defense);
        } else if current.is_some() {
            commands.entity(entity).remove::<TowerDefenseTarget>();
        }
    }
}

fn collect_damage(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    spatial: Res<SpatialIndex>,
    factions: Query<&Faction>,
    live_targets: Query<(&Position, &Allegiance, Option<&Health>), Or<(With<Tower>, With<Minion>)>>,
    mut attackers: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            &mut Attack,
            &mut EnergyStore,
            &Vision,
            Option<&CombatTarget>,
        ),
        With<Minion>,
    >,
    mut alerts: ResMut<PendingTowerAlerts>,
    config: Res<SimulationConfig>,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, position, faction, mut attack, mut energy, vision, selected) in &mut attackers {
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
        let target = selected.and_then(|selected| {
            spatial
                .nearby(position.0, vision.radius.max(0.0))
                .find(|entry| {
                    entry.entity == selected.0
                        && entry.faction != faction.0
                        && position.0.distance(entry.position)
                            <= attack.range + if entry.tower { 2.0 } else { 0.0 }
                })
        });
        if let Some(target) = target {
            let Ok((live_position, live_owner, health)) = live_targets.get(target.entity) else {
                continue;
            };
            let distance = position.0.distance(live_position.0);
            if live_owner.0 == faction.0
                || health.is_some_and(|health| health.current <= 0.0)
                || distance > vision.radius.max(0.0)
                || distance > attack.range + if target.tower { 2.0 } else { 0.0 }
            {
                continue;
            }
            if target.tower {
                alerts.0.push(TowerAlert {
                    tower: target.entity,
                    attacker: entity,
                });
            }
            commands.spawn((
                Name::new("Energy projectile"),
                EntityKind::EnergyProjectile,
                MatchEntity,
                Position(position.0),
                Allegiance(faction.0),
                EnergyProjectile {
                    target: target.entity,
                    damage: attack.damage,
                    speed: config.projectile_speed,
                    remaining: config.projectile_lifetime,
                },
            ));
            commands.entity(entity).insert(ShootingPause(1.0));
            attack.remaining = attack.cooldown;
            energy.current -= config.attack_energy_cost;
        }
    }
}

fn resolve_damage(
    mut commands: Commands,
    mut pending: ResMut<PendingDamage>,
    mut units: Query<
        (&mut Health, &Position, Option<&RawElement>),
        Or<(With<Tower>, With<Minion>)>,
    >,
    ids: Query<&SimulationId>,
) {
    let mut hits: Vec<_> = pending.0.drain().collect();
    hits.sort_by_key(|(entity, _)| ids.get(*entity).map_or(entity.to_bits(), |id| id.0));
    for (entity, contributions) in hits {
        let Ok((mut health, position, cargo)) = units.get_mut(entity) else {
            continue;
        };
        health.current -= contributions.iter().sum::<f32>();
        if health.current > 0.0 {
            continue;
        }
        if let Some(cargo) = cargo
            && cargo.0 > 0
        {
            crate::gathering::drop_cargo(&mut commands, position.0, cargo.0);
        }
        commands.entity(entity).despawn();
    }
}

fn check_victory(
    towers: Query<&Allegiance, With<Tower>>,
    factions: Query<&Faction>,
    mut state: ResMut<MatchState>,
) {
    let mut ownership = towers.iter();
    let Some(first) = ownership.next() else {
        state.winner = None;
        state.paused = true;
        return;
    };
    if ownership.next().is_none()
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
        let controlled: Vec<_> = app
            .world_mut()
            .query_filtered::<&Tower, With<PlayerControlled>>()
            .iter(app.world())
            .map(|tower| tower.slot)
            .collect();
        assert_eq!(controlled, vec![0]);
        assert_eq!(app.world().resource::<SpawnSequence>().0, 20);
        for attack in app
            .world_mut()
            .query_filtered::<&Attack, With<Minion>>()
            .iter(app.world())
        {
            assert_eq!(attack.range, 6.0);
        }
        let mut ids = Vec::new();
        for (id, energy, raw, vision, state) in app
            .world_mut()
            .query_filtered::<(
                &SimulationId,
                &EnergyStore,
                &RawElement,
                &Vision,
                &GatheringState,
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
    fn stock_expands_radius_without_spending_or_replacing_lost_minions() {
        let mut app = app();
        let tower = tower(&mut app);
        app.world_mut().get_mut::<RawElement>(tower).unwrap().0 = 17;
        let lost = app
            .world_mut()
            .query_filtered::<Entity, With<Minion>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().despawn(lost);
        let mut schedule = Schedule::default();
        schedule.add_systems(grow_tower_radius);
        for _ in 0..20 {
            schedule.run(app.world_mut());
        }
        assert_eq!(count::<Minion>(&mut app), 14);
        assert_eq!(app.world().get::<RawElement>(tower).unwrap().0, 17);
        let emitter = app.world().get::<InfluenceEmitter>(tower).unwrap();
        assert_eq!(emitter.radius, 32.0);
        assert_eq!(emitter.strength, 4.0);
    }

    #[test]
    fn destroyed_tower_drops_stock_and_preserves_existing_army() {
        let mut app = app();
        let tower = tower(&mut app);
        let old_owner = app.world().get::<Allegiance>(tower).unwrap().0;
        let army: Vec<_> = app
            .world_mut()
            .query::<(Entity, &OriginTower)>()
            .iter(app.world())
            .filter(|(_, origin)| origin.0 == tower)
            .map(|(entity, _)| entity)
            .collect();
        app.world_mut().get_mut::<RawElement>(tower).unwrap().0 = 4;
        let before = total(&mut app);
        let deposits = count::<RawElementDeposit>(&mut app);
        app.world_mut()
            .resource_mut::<PendingDamage>()
            .0
            .insert(tower, [0.0, 200.0, 200.0, 0.0, 0.0]);
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_damage);
        schedule.run(app.world_mut());
        assert!(app.world().get_entity(tower).is_err());
        assert_eq!(count::<Tower>(&mut app), 4);
        assert_eq!(count::<RawElementDeposit>(&mut app), deposits + 1);
        assert_eq!(total(&mut app), before);
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
    fn projectiles_home_impact_once_and_fizzle_for_invalid_targets_or_timeout() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f32(0.25));
        let target = app
            .world_mut()
            .spawn((
                Minion,
                Position(Vec2::X * 10.0),
                Allegiance(factions[1]),
                Health {
                    current: 30.0,
                    max: 30.0,
                },
            ))
            .id();
        let shot = app
            .world_mut()
            .spawn((
                MatchEntity,
                Position(Vec2::ZERO),
                Allegiance(factions[0]),
                EnergyProjectile {
                    target,
                    damage: 7.0,
                    speed: 4.0,
                    remaining: 4.0,
                },
            ))
            .id();
        let mut advance = Schedule::default();
        advance.add_systems((clear_pending_damage, advance_projectiles).chain());
        advance.run(app.world_mut());
        assert_eq!(app.world().get::<Position>(shot).unwrap().0, Vec2::X);
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(app.world().get::<Health>(target).unwrap().current, 30.0);
        // The shot follows the target's new position and lands without overshoot.
        app.world_mut().get_mut::<Position>(target).unwrap().0 = Vec2::new(1.0, 0.5);
        advance.run(app.world_mut());
        assert!(app.world().get_entity(shot).is_err());
        assert_eq!(app.world().resource::<PendingDamage>().0[&target][0], 7.0);
        let mut resolve = Schedule::default();
        resolve.add_systems(resolve_damage);
        resolve.run(app.world_mut());
        assert_eq!(app.world().get::<Health>(target).unwrap().current, 23.0);
        advance.run(app.world_mut());
        assert!(app.world().resource::<PendingDamage>().0.is_empty());

        for mode in 0..4 {
            app.world_mut().get_mut::<Allegiance>(target).unwrap().0 =
                if mode == 0 { factions[0] } else { factions[1] };
            app.world_mut().get_mut::<Health>(target).unwrap().current =
                if mode == 1 { 0.0 } else { 23.0 };
            let missing = app.world_mut().spawn_empty().id();
            app.world_mut().despawn(missing);
            let shot = app
                .world_mut()
                .spawn((
                    Position(Vec2::splat(100.0)),
                    Allegiance(factions[0]),
                    EnergyProjectile {
                        target: if mode == 2 { missing } else { target },
                        damage: 7.0,
                        speed: 4.0,
                        remaining: if mode == 3 { 0.1 } else { 4.0 },
                    },
                ))
                .id();
            advance.run(app.world_mut());
            assert!(app.world().get_entity(shot).is_err());
            assert!(app.world().resource::<PendingDamage>().0.is_empty());
        }
    }

    #[test]
    fn combat_selects_closest_visible_enemy_then_stable_id() {
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
                Position(center + Vec2::X * 2.5),
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
        combat.add_systems((rebuild_index, select_combat_targets, collect_damage).chain());
        combat.run(app.world_mut());
        let shots: Vec<_> = app
            .world_mut()
            .query::<&EnergyProjectile>()
            .iter(app.world())
            .copied()
            .collect();
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].target, preferred);
        assert_eq!(shots[0].damage, 10.0);
        let config = app.world().resource::<SimulationConfig>();
        assert_eq!(shots[0].speed, config.projectile_speed);
        assert_eq!(shots[0].remaining, config.projectile_lifetime);
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 4);
        assert_eq!(app.world().get::<Attack>(attacker).unwrap().remaining, 0.8);

        // Cooldown prevents another strike even with eligible targets present.
        combat.run(app.world_mut());
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 4);

        // A visible minion outside attack range must not prevent tower fallback.
        for minion in [preferred, farther, tied] {
            app.world_mut().get_mut::<Position>(minion).unwrap().0 = center + Vec2::X * 3.0;
        }
        app.world_mut()
            .get_mut::<Attack>(attacker)
            .unwrap()
            .remaining = 0.0;
        combat.run(app.world_mut());
        assert!(
            app.world_mut()
                .query::<&EnergyProjectile>()
                .iter(app.world())
                .any(|shot| shot.target == tower)
        );
        assert_eq!(count::<EnergyProjectile>(&mut app), 2);
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 3);

        // Without a visible target, neither energy nor cooldown is spent.
        for minion in [preferred, farther, tied] {
            app.world_mut().get_mut::<Position>(minion).unwrap().0 = center + Vec2::X * 4.0;
        }
        app.world_mut().get_mut::<Position>(tower).unwrap().0 = center + Vec2::X * 4.0;
        app.world_mut()
            .get_mut::<Attack>(attacker)
            .unwrap()
            .remaining = 0.0;
        combat.run(app.world_mut());
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 3);
        assert_eq!(app.world().get::<Attack>(attacker).unwrap().remaining, 0.0);
        app.world_mut().despawn(attacker);
        assert_eq!(count::<EnergyProjectile>(&mut app), 2);
        // A shot has no ownership relationship to the entity that fired it.
        app.world_mut().entity_mut(preferred).insert(Health {
            current: 30.0,
            max: 30.0,
        });
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f32(0.25));
        let mut flight = Schedule::default();
        flight.add_systems((clear_pending_damage, advance_projectiles, resolve_damage).chain());
        flight.run(app.world_mut());
        assert_eq!(app.world().get::<Health>(preferred).unwrap().current, 20.0);
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
        combat.add_systems((rebuild_index, select_combat_targets, collect_damage).chain());
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
    fn combat_uses_actual_inclusive_vision_and_pauses_gathering_movement() {
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
                RawElement(1),
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
        combat.add_systems((rebuild_index, select_combat_targets, collect_damage).chain());
        combat.run(app.world_mut());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 5);
        app.world_mut().get_mut::<Position>(target).unwrap().0 = Vec2::X;
        combat.run(app.world_mut());
        assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 4);
        assert!(
            app.world_mut()
                .query::<&EnergyProjectile>()
                .iter(app.world())
                .any(|shot| shot.target == target)
        );
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 30.0));
        let mut movement = Schedule::default();
        movement.add_systems(move_minions);
        movement.run(app.world_mut());
        assert_eq!(app.world().get::<Position>(attacker).unwrap().0, Vec2::ZERO);
        assert!(app.world().get::<ShootingPause>(attacker).unwrap().0 < 1.0);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f32(1.0));
        movement.run(app.world_mut());
        movement.run(app.world_mut());
        assert!(app.world().get::<Position>(attacker).unwrap().0.y > 0.0);
        assert_eq!(
            app.world().get::<Movement>(attacker).unwrap().target,
            Vec2::Y * 10.0
        );
    }

    #[test]
    fn defense_alerts_use_influence_vision_and_deterministic_closest_attacker() {
        let mut app = app();
        let owners = app.world().resource::<FactionEntities>().0.clone();
        let center = Vec2::splat(1000.0);
        let tower = app
            .world_mut()
            .spawn((
                Tower { slot: 99 },
                Position(center),
                Allegiance(owners[0]),
                InfluenceEmitter {
                    radius: 5.0,
                    strength: 1.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();
        let spawn_enemy = |app: &mut App, x: f32, id: u64| {
            app.world_mut()
                .spawn((
                    Minion,
                    Position(center + Vec2::X * x),
                    Allegiance(owners[1]),
                    SimulationId(id),
                    Health {
                        current: 30.0,
                        max: 30.0,
                    },
                ))
                .id()
        };
        let farther = spawn_enemy(&mut app, 8.0, 50);
        let tied = spawn_enemy(&mut app, -6.0, 52);
        let preferred = spawn_enemy(&mut app, 6.0, 51);
        let spawn_defender = |app: &mut App, x: f32, vision: f32| {
            app.world_mut()
                .spawn((
                    Minion,
                    Position(center + Vec2::X * x),
                    Allegiance(owners[0]),
                    Vision { radius: vision },
                    RawElement(1),
                    EnergyStore {
                        current: 5,
                        capacity: 5,
                    },
                    Attack {
                        damage: 10.0,
                        range: 2.0,
                        cooldown: 1.0,
                        remaining: 0.0,
                    },
                    Movement {
                        speed: 4.0,
                        target: center - Vec2::Y * 10.0,
                        retarget_in: 0.0,
                    },
                ))
                .id()
        };
        let defender = spawn_defender(&mut app, 0.0, 10.0);
        let outside = spawn_defender(&mut app, 5.1, 20.0);
        let blind = spawn_defender(&mut app, 0.0, 2.0);
        app.world_mut()
            .resource_mut::<PendingTowerAlerts>()
            .0
            .extend([farther, tied, preferred].map(|attacker| TowerAlert { tower, attacker }));
        let mut alerts = Schedule::default();
        alerts.add_systems(process_tower_alerts);
        alerts.run(app.world_mut());
        assert_eq!(
            app.world()
                .get::<TowerDefenseTarget>(defender)
                .unwrap()
                .attacker,
            preferred
        );
        assert!(app.world().get::<TowerDefenseTarget>(outside).is_none());
        assert!(app.world().get::<TowerDefenseTarget>(blind).is_none());
        assert!(app.world().resource::<PendingTowerAlerts>().0.is_empty());
        let mut select = Schedule::default();
        select.add_systems((rebuild_index, select_combat_targets).chain());
        select.run(app.world_mut());
        assert_eq!(
            app.world().get::<CombatTarget>(defender).unwrap().0,
            preferred
        );
        assert_eq!(
            app.world().get::<Movement>(defender).unwrap().target,
            center + Vec2::X * 4.0
        );
        assert_eq!(app.world().get::<RawElement>(defender).unwrap().0, 1);
        // A valid current alert competes with later alerts using the same ordering.
        app.world_mut()
            .resource_mut::<PendingTowerAlerts>()
            .0
            .push(TowerAlert {
                tower,
                attacker: farther,
            });
        alerts.run(app.world_mut());
        assert_eq!(
            app.world()
                .get::<TowerDefenseTarget>(defender)
                .unwrap()
                .attacker,
            preferred
        );
        // Leaving vision cancels defense. Cargo then prevents chasing any visible enemy.
        app.world_mut().get_mut::<Position>(preferred).unwrap().0 = center + Vec2::X * 11.0;
        select.run(app.world_mut());
        assert!(app.world().get::<TowerDefenseTarget>(defender).is_none());
        assert!(app.world().get::<CombatTarget>(defender).is_none());
        assert_eq!(app.world().get::<RawElement>(defender).unwrap().0, 1);
        // Empty minions pursue; zero energy suspends pursuit until refilled.
        app.world_mut().get_mut::<RawElement>(defender).unwrap().0 = 0;
        select.run(app.world_mut());
        assert_eq!(app.world().get::<CombatTarget>(defender).unwrap().0, tied);
        app.world_mut()
            .get_mut::<EnergyStore>(defender)
            .unwrap()
            .current = 0;
        select.run(app.world_mut());
        assert!(app.world().get::<CombatTarget>(defender).is_none());
        app.world_mut()
            .get_mut::<EnergyStore>(defender)
            .unwrap()
            .current = 1;
        select.run(app.world_mut());
        assert_eq!(app.world().get::<CombatTarget>(defender).unwrap().0, tied);
    }

    #[test]
    fn firing_revalidates_shared_target_without_spending_energy() {
        let mut app = app();
        let owners = app.world().resource::<FactionEntities>().0.clone();
        let center = Vec2::splat(1000.0);
        let target = app
            .world_mut()
            .spawn((
                Minion,
                Position(center + Vec2::X),
                Allegiance(owners[1]),
                Health {
                    current: 30.0,
                    max: 30.0,
                },
            ))
            .id();
        let attacker = app
            .world_mut()
            .spawn((
                Minion,
                Position(center),
                Allegiance(owners[0]),
                Vision { radius: 10.0 },
                EnergyStore {
                    current: 5,
                    capacity: 5,
                },
                Attack {
                    damage: 10.0,
                    range: 2.0,
                    cooldown: 1.0,
                    remaining: 0.0,
                },
            ))
            .id();
        let mut select = Schedule::default();
        select.add_systems((rebuild_index, select_combat_targets).chain());
        let mut fire = Schedule::default();
        fire.add_systems(collect_damage);
        for mode in 0..4 {
            app.world_mut().entity_mut(target).insert((
                Allegiance(owners[1]),
                Position(center + Vec2::X),
                Health {
                    current: 30.0,
                    max: 30.0,
                },
            ));
            select.run(app.world_mut());
            assert_eq!(app.world().get::<CombatTarget>(attacker).unwrap().0, target);
            match mode {
                0 => app.world_mut().get_mut::<Allegiance>(target).unwrap().0 = owners[0],
                1 => app.world_mut().get_mut::<Health>(target).unwrap().current = 0.0,
                2 => {
                    app.world_mut().get_mut::<Position>(target).unwrap().0 = center + Vec2::X * 11.0
                }
                _ => {
                    app.world_mut().despawn(target);
                }
            }
            fire.run(app.world_mut());
            assert_eq!(app.world().get::<EnergyStore>(attacker).unwrap().current, 5);
            assert_eq!(app.world().get::<Attack>(attacker).unwrap().remaining, 0.0);
            assert_eq!(count::<EnergyProjectile>(&mut app), 0);
        }
    }

    #[test]
    fn fixed_schedule_applies_combat_cargo_and_defense_priorities() {
        let mut app = app();
        let owners = app.world().resource::<FactionEntities>().0.clone();
        let old: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, Or<(With<Minion>, With<RawElementDeposit>)>>()
            .iter(app.world())
            .collect();
        for entity in old {
            app.world_mut().despawn(entity);
        }
        let tower = app
            .world_mut()
            .spawn((
                Tower { slot: 99 },
                Position(Vec2::X * -20.0),
                Allegiance(owners[0]),
                RawElement(0),
                InfluenceEmitter {
                    radius: 30.0,
                    strength: 1.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();
        let enemy = app
            .world_mut()
            .spawn((
                Minion,
                Position(Vec2::X * 10.0),
                Allegiance(owners[1]),
                Health {
                    current: 30.0,
                    max: 30.0,
                },
            ))
            .id();
        app.world_mut().spawn((
            RawElementDeposit,
            Position(Vec2::Y * 10.0),
            RawElement(1),
            SimulationId(500),
        ));
        app.world_mut().spawn((
            Trail {
                trip: 9,
                sequence: 0,
                remaining: 30.0,
            },
            Position(Vec2::Y * 5.0),
            Allegiance(owners[0]),
        ));
        let minion = app
            .world_mut()
            .spawn((
                Minion,
                SimulationId(501),
                Position(Vec2::ZERO),
                Allegiance(owners[0]),
                Vision { radius: 20.0 },
                RawElement(0),
                GatheringState::Exploring,
                EnergyStore {
                    current: 5,
                    capacity: 5,
                },
                Attack {
                    damage: 10.0,
                    range: 2.0,
                    cooldown: 1.0,
                    remaining: 0.0,
                },
                Movement {
                    speed: 0.0,
                    target: Vec2::ZERO,
                    retarget_in: 0.0,
                },
            ))
            .id();
        step(&mut app, 1);
        assert_eq!(
            app.world().get::<Movement>(minion).unwrap().target,
            Vec2::X * 8.0
        );
        app.world_mut().get_mut::<RawElement>(minion).unwrap().0 = 1;
        app.world_mut()
            .entity_mut(minion)
            .insert(GatheringState::Returning { tower: Some(tower) });
        step(&mut app, 1);
        assert_eq!(
            app.world().get::<Movement>(minion).unwrap().target,
            Vec2::X * -20.0
        );
        assert!(app.world().get::<CombatTarget>(minion).is_none());
        app.world_mut()
            .entity_mut(minion)
            .insert(TowerDefenseTarget {
                tower,
                attacker: enemy,
            });
        step(&mut app, 1);
        assert_eq!(
            app.world().get::<Movement>(minion).unwrap().target,
            Vec2::X * 8.0
        );
        assert_eq!(app.world().get::<RawElement>(minion).unwrap().0, 1);
        app.world_mut()
            .get_mut::<EnergyStore>(minion)
            .unwrap()
            .current = 0;
        step(&mut app, 1);
        assert_eq!(
            app.world().get::<Movement>(minion).unwrap().target,
            Vec2::X * -20.0
        );
    }

    #[test]
    fn firing_at_tower_alerts_defenders_before_projectile_impact() {
        let mut app = app();
        let owners = app.world().resource::<FactionEntities>().0.clone();
        let center = Vec2::splat(1000.0);
        let tower = app
            .world_mut()
            .spawn((
                Tower { slot: 99 },
                Position(center),
                Allegiance(owners[0]),
                InfluenceEmitter {
                    radius: 10.0,
                    strength: 1.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();
        let defender = app
            .world_mut()
            .spawn((
                Minion,
                Position(center - Vec2::X),
                Allegiance(owners[0]),
                Vision { radius: 10.0 },
            ))
            .id();
        let attacker = app
            .world_mut()
            .spawn((
                Minion,
                Position(center + Vec2::X * 3.0),
                Allegiance(owners[1]),
                Vision { radius: 10.0 },
                Health {
                    current: 30.0,
                    max: 30.0,
                },
                EnergyStore {
                    current: 5,
                    capacity: 5,
                },
                Attack {
                    damage: 10.0,
                    range: 2.0,
                    cooldown: 1.0,
                    remaining: 0.0,
                },
            ))
            .id();
        let mut combat = Schedule::default();
        combat.add_systems(
            (
                rebuild_index,
                select_combat_targets,
                collect_damage,
                process_tower_alerts,
            )
                .chain(),
        );
        combat.run(app.world_mut());
        assert_eq!(app.world().get::<CombatTarget>(attacker).unwrap().0, tower);
        assert_eq!(
            app.world()
                .get::<TowerDefenseTarget>(defender)
                .expect("a visible living attacker should trigger defense before projectile impact")
                .attacker,
            attacker
        );
        assert_eq!(app.world().get::<Health>(tower).unwrap().current, 100.0);
        assert!(app.world().resource::<PendingDamage>().0.is_empty());
        assert_eq!(count::<EnergyProjectile>(&mut app), 1);
    }

    #[test]
    fn victory_requires_one_surviving_tower_and_empty_arena_is_a_draw() {
        let mut app = app();
        let towers: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<Tower>>()
            .iter(app.world())
            .collect();
        let mut victory = Schedule::default();
        victory.add_systems(check_victory);
        victory.run(app.world_mut());
        assert_eq!(app.world().resource::<MatchState>().winner, None);
        for entity in &towers[1..] {
            app.world_mut().despawn(*entity);
        }
        let owner = app.world().get::<Allegiance>(towers[0]).unwrap().0;
        let winner = app.world().get::<Faction>(owner).unwrap().index;
        victory.run(app.world_mut());
        assert_eq!(app.world().resource::<MatchState>().winner, Some(winner));
        assert!(app.world().resource::<MatchState>().paused);
        app.world_mut().despawn(towers[0]);
        victory.run(app.world_mut());
        assert_eq!(app.world().resource::<MatchState>().winner, None);
        assert!(app.world().resource::<MatchState>().paused);
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
            .resource_mut::<PendingTowerAlerts>()
            .0
            .push(TowerAlert {
                tower: old[0],
                attacker: old[1],
            });
        app.world_mut()
            .resource_mut::<MatchState>()
            .restart_requested = true;
        app.world_mut().run_schedule(PreUpdate);
        assert_eq!(layout(&mut app), original);
        assert_eq!(count::<Minion>(&mut app), 15);
        assert_eq!(count::<Tower>(&mut app), 5);
        assert_eq!(count::<Trail>(&mut app), 0);
        assert!(app.world().resource::<PendingTowerAlerts>().0.is_empty());
        assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
        assert_eq!(app.world().resource::<EnergyClock>().pulses, 0);
        assert_eq!(app.world().resource::<EnergyClock>().elapsed, 0.0);
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
            assert!(minions <= 15);
            assert!(count::<Tower>(&mut app) <= 5);
            assert!(count::<RawElementDeposit>(&mut app) as u32 <= initial);
            assert!(count::<Trail>(&mut app) <= 500 * 100);
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
