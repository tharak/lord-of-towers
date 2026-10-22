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
                    SimSet::Production,
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
            .add_systems(FixedUpdate, produce_minions.in_set(SimSet::Production))
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
                ElementInventory::default(),
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
                ElementStats::default(),
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
    stats: ElementStats,
) {
    let direction = random.direction();
    let location = (position + direction * 2.8).clamp(
        Vec2::splat(-config.half_extent),
        Vec2::splat(config.half_extent),
    );
    let stable_id = sequence.0;
    sequence.0 += 1;
    let health = config.minion_health * stats.multiplier(stats.earth);
    commands.spawn((
        (
            SimulationId(stable_id),
            stats,
            EnergyStore {
                current: config.initial_energy.min(config.energy_capacity),
                capacity: config.energy_capacity,
            },
            ElementInventory::default(),
            Vision {
                radius: config.vision_radius * stats.multiplier(stats.air),
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
                current: health,
                max: health,
            },
            Movement {
                speed: config.minion_speed * stats.multiplier(stats.water),
                target: location + direction,
                retarget_in: 0.0,
            },
            Attack {
                damage: 10.0 * stats.multiplier(stats.fire),
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

/// Each birth consumes the oldest five units without changing existing minions.
fn produce_minions(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut random: ResMut<RandomStream>,
    mut sequence: ResMut<SpawnSequence>,
    mut economy: ResMut<ElementEconomy>,
    mut towers: Query<
        (
            Entity,
            &SimulationId,
            &Position,
            &Allegiance,
            &Health,
            &mut ElementInventory,
        ),
        With<Tower>,
    >,
) {
    let mut ordered: Vec<_> = towers
        .iter()
        .map(|(entity, id, ..)| (id.0, entity))
        .collect();
    ordered.sort_by_key(|(id, _)| *id);
    for (_, entity) in ordered {
        let Ok((tower, _, position, owner, health, mut inventory)) = towers.get_mut(entity) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        while inventory.total() >= ELEMENTS_PER_MINION {
            let recipe: Vec<_> = inventory.units.drain(..ELEMENTS_PER_MINION).collect();
            for element in &recipe {
                economy.consumed[*element as usize] += 1;
            }
            spawn_minion(
                &mut commands,
                &config,
                &mut random,
                &mut sequence,
                tower,
                position.0,
                *owner,
                ElementStats::from_recipe(recipe),
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
            Option<&ElementInventory>,
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
        let carrying = cargo.is_some_and(|cargo| !cargo.is_empty());
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
        (&mut Health, &Position, Option<&ElementInventory>),
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
            && !cargo.is_empty()
        {
            crate::gathering::drop_cargo(&mut commands, position.0, cargo.clone());
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
