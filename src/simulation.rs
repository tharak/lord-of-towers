//! Deterministic, renderer-independent match systems.
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
                    SimSet::Influence,
                    SimSet::Victory,
                )
                    .chain()
                    .run_if(simulation_running),
            )
            .add_systems(Startup, setup_match)
            .add_systems(Update, restart_match)
            .add_systems(
                FixedUpdate,
                (advance_clock, spawn_minions).chain().in_set(SimSet::Spawn),
            )
            .add_systems(FixedUpdate, rebuild_index.in_set(SimSet::Index))
            .add_systems(FixedUpdate, choose_targets.in_set(SimSet::Target))
            .add_systems(FixedUpdate, move_minions.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                (rebuild_index, collect_damage)
                    .chain()
                    .in_set(SimSet::Combat),
            )
            .add_systems(FixedUpdate, resolve_damage.in_set(SimSet::Capture))
            .add_systems(FixedUpdate, check_victory.in_set(SimSet::Victory));
    }
}

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
) {
    create_match(&mut commands, &config, &mut factions, &mut random);
}

fn create_match(
    commands: &mut Commands,
    config: &SimulationConfig,
    factions: &mut FactionEntities,
    random: &mut RandomStream,
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
        commands.spawn((
            Name::new(format!("{name} tower")),
            EntityKind::Tower,
            MatchEntity,
            Tower { slot: index },
            Allegiance(faction),
            Position(position),
            Health {
                current: config.tower_health,
                max: config.tower_health,
            },
            Spawner {
                elapsed: 0.0,
                interval: config.spawn_interval,
                cap: config.population_cap,
            },
            InfluenceEmitter {
                radius: 15.0,
                strength: 4.0,
            },
        ));
    }
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
    if let Some(mut field) = field {
        field.clear();
    }
    create_match(&mut commands, &config, &mut factions, &mut random);
}

fn advance_clock(time: Res<Time<Fixed>>, mut state: ResMut<MatchState>) {
    state.elapsed += simulation_delta(&time, &state);
}

fn spawn_minions(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    config: Res<SimulationConfig>,
    mut random: ResMut<RandomStream>,
    mut towers: Query<(Entity, &Position, &Allegiance, &mut Spawner), With<Tower>>,
    minions: Query<&OriginTower, With<Minion>>,
) {
    let dt = simulation_delta(&time, &state);
    let mut counts = HashMap::<Entity, usize>::new();
    for origin in &minions {
        *counts.entry(origin.0).or_default() += 1;
    }
    // Stable order makes seeded spawn variation reproducible after restart.
    let mut order: Vec<_> = towers.iter().map(|(entity, ..)| entity).collect();
    order.sort_by_key(|entity| entity.to_bits());
    for entity in order {
        let Ok((tower, position, allegiance, mut spawner)) = towers.get_mut(entity) else {
            continue;
        };
        spawner.elapsed += dt;
        let interval = spawner.interval.max(0.01);
        let count = counts.entry(tower).or_default();
        if *count >= spawner.cap {
            spawner.elapsed = spawner.elapsed.min(interval);
            continue;
        }
        while spawner.elapsed >= interval && *count < spawner.cap {
            spawner.elapsed -= interval;
            *count += 1;
            let direction = random.direction();
            let location = (position.0 + direction * 2.8).clamp(
                Vec2::splat(-config.half_extent),
                Vec2::splat(config.half_extent),
            );
            commands.spawn((
                Name::new("Minion"),
                EntityKind::Minion,
                MatchEntity,
                Minion,
                *allegiance,
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
            ));
        }
    }
}

fn rebuild_index(
    mut spatial: ResMut<SpatialIndex>,
    units: Query<(Entity, &Position, &Allegiance, Option<&Tower>)>,
) {
    spatial.0.clear();
    for (entity, position, faction, tower) in &units {
        spatial
            .0
            .entry(SpatialIndex::cell(position.0))
            .or_default()
            .push(Neighbor {
                entity,
                position: position.0,
                faction: faction.0,
                tower: tower.is_some(),
            });
    }
    for bucket in spatial.0.values_mut() {
        bucket.sort_by_key(|entry| entry.entity.to_bits());
    }
}

fn choose_targets(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    config: Res<SimulationConfig>,
    spatial: Res<SpatialIndex>,
    field: Option<Res<InfluenceField>>,
    factions: Query<&Faction>,
    towers: Query<(&Position, &Allegiance), With<Tower>>,
    mut minions: Query<(Entity, &Position, &Allegiance, &mut Movement), With<Minion>>,
    mut random: ResMut<RandomStream>,
) {
    let dt = simulation_delta(&time, &state);
    let mut order: Vec<_> = minions.iter().map(|(entity, ..)| entity).collect();
    order.sort_by_key(|entity| entity.to_bits());
    for entity in order {
        let Ok((_, position, faction, mut movement)) = minions.get_mut(entity) else {
            continue;
        };
        movement.retarget_in -= dt;
        if movement.retarget_in > 0.0 {
            continue;
        }
        movement.retarget_in = 0.35 + random.unit() * 0.25;
        if let Some(enemy) = spatial
            .nearby(position.0, 10.0)
            .filter(|entry| entry.faction != faction.0)
            .min_by(|a, b| {
                position
                    .0
                    .distance_squared(a.position)
                    .total_cmp(&position.0.distance_squared(b.position))
                    .then(a.entity.to_bits().cmp(&b.entity.to_bits()))
            })
        {
            movement.target = enemy.position;
            continue;
        }
        let destination = towers
            .iter()
            .filter(|(_, allegiance)| allegiance.0 != faction.0)
            .min_by(|(a, _), (b, _)| {
                position
                    .0
                    .distance_squared(a.0)
                    .total_cmp(&position.0.distance_squared(b.0))
            })
            .map(|(position, _)| position.0)
            .unwrap_or(Vec2::ZERO);
        let forward = (destination - position.0).normalize_or_zero();
        let faction_index = factions.get(faction.0).map_or(0, |f| f.index);
        // Prefer weakly supported ground while always progressing toward a rival tower.
        let mut best = destination;
        let mut score = f32::NEG_INFINITY;
        for _ in 0..5 {
            let candidate = (position.0
                + (forward + random.direction() * 0.65).normalize_or_zero() * 7.0)
                .clamp(
                    Vec2::splat(-config.half_extent),
                    Vec2::splat(config.half_extent),
                );
            let support = field
                .as_ref()
                .map_or(0.0, |field| field.sample(candidate)[faction_index]);
            let progress = position.0.distance(destination) - candidate.distance(destination);
            let value = progress - support.min(8.0) * 0.6 + random.unit() * 0.3;
            if value > score {
                score = value;
                best = candidate;
            }
        }
        movement.target = best;
    }
}

fn move_minions(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    config: Res<SimulationConfig>,
    spatial: Res<SpatialIndex>,
    mut minions: Query<(Entity, &mut Position, &Movement, &Allegiance, &Attack), With<Minion>>,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, mut position, movement, faction, attack) in &mut minions {
        let engaged = spatial.nearby(position.0, attack.range + 2.0).any(|entry| {
            entry.faction != faction.0
                && position.0.distance(entry.position)
                    <= attack.range + if entry.tower { 2.0 } else { 0.0 }
        });
        let offset = movement.target - position.0;
        let mut velocity = if engaged {
            Vec2::ZERO
        } else {
            offset.normalize_or_zero() * movement.speed.min(offset.length() / dt.max(0.0001))
        };
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
    mut attackers: Query<(&Position, &Allegiance, &mut Attack), With<Minion>>,
) {
    pending.0.clear();
    let dt = simulation_delta(&time, &state);
    for (position, faction, mut attack) in &mut attackers {
        attack.remaining = (attack.remaining - dt).max(0.0);
        if attack.remaining > 0.0 {
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
                    && position.0.distance(entry.position)
                        <= attack.range + if entry.tower { 2.0 } else { 0.0 }
            })
            .min_by(|a, b| {
                position
                    .0
                    .distance_squared(a.position)
                    .total_cmp(&position.0.distance_squared(b.position))
                    .then(a.entity.to_bits().cmp(&b.entity.to_bits()))
            });
        if let Some(target) = target {
            pending.0.entry(target.entity).or_default()[faction.1] += attack.damage;
            attack.remaining = attack.cooldown;
        }
    }
}

fn resolve_damage(
    mut commands: Commands,
    mut pending: ResMut<PendingDamage>,
    factions: Res<FactionEntities>,
    mut units: Query<
        (&mut Health, &mut Allegiance, Option<&mut Spawner>),
        Or<(With<Tower>, With<Minion>)>,
    >,
) {
    for (entity, contributions) in pending.0.drain() {
        let Ok((mut health, mut allegiance, spawner)) = units.get_mut(entity) else {
            continue;
        };
        health.current -= contributions.iter().sum::<f32>();
        if health.current > 0.0 {
            continue;
        }
        if let Some(mut spawner) = spawner {
            let mut winner = 0;
            for index in 1..5 {
                if contributions[index] > contributions[winner] {
                    winner = index;
                }
            }
            if let Some(faction) = factions.0.get(winner) {
                allegiance.0 = *faction;
                health.current = health.max;
                spawner.elapsed = 0.0;
            }
        } else {
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
    if ownership.all(|owner| owner == first) {
        if let Ok(faction) = factions.get(first.0) {
            state.winner = Some(faction.index);
            state.paused = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SimulationPlugin);
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

    #[test]
    fn spawning_obeys_time_and_origin_cap() {
        let mut app = app();
        for mut spawn in app
            .world_mut()
            .query::<&mut Spawner>()
            .iter_mut(app.world_mut())
        {
            spawn.cap = 1;
        }
        step(&mut app, 59);
        assert_eq!(count::<Minion>(&mut app), 0);
        step(&mut app, 2);
        assert_eq!(count::<Minion>(&mut app), 5);
        step(&mut app, 120);
        assert_eq!(count::<Minion>(&mut app), 5);
    }

    #[test]
    fn pause_hidden_tab_and_victory_freeze_fixed_simulation() {
        let mut app = app();
        app.world_mut().resource_mut::<MatchState>().paused = true;
        step(&mut app, 90);
        assert_eq!(count::<Minion>(&mut app), 0);
        assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
        {
            let mut state = app.world_mut().resource_mut::<MatchState>();
            state.paused = false;
            state.browser_hidden = true;
        }
        step(&mut app, 90);
        assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
        {
            let mut state = app.world_mut().resource_mut::<MatchState>();
            state.browser_hidden = false;
            state.winner = Some(0);
        }
        step(&mut app, 90);
        assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
    }

    #[test]
    fn restart_removes_old_match_references_and_resets_clock() {
        let mut app = app();
        step(&mut app, 70);
        let old = app.world().resource::<FactionEntities>().0.clone();
        app.world_mut()
            .resource_mut::<MatchState>()
            .restart_requested = true;
        app.world_mut().run_schedule(Update);
        assert_eq!(count::<Minion>(&mut app), 0);
        assert_eq!(count::<Tower>(&mut app), 5);
        assert_eq!(count::<Faction>(&mut app), 5);
        assert_eq!(app.world().resource::<MatchState>().elapsed, 0.0);
        for entity in old {
            assert!(app.world().get_entity(entity).is_err());
        }
    }

    #[test]
    fn simultaneous_damage_kills_both_attackers() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        for faction in factions.iter().take(2) {
            app.world_mut().spawn((
                Minion,
                Position(Vec2::ZERO),
                Allegiance(*faction),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
                Attack {
                    damage: 10.0,
                    range: 1.6,
                    cooldown: 0.8,
                    remaining: 0.0,
                },
            ));
        }
        step(&mut app, 1);
        assert_eq!(count::<Minion>(&mut app), 0);
    }

    #[test]
    fn capture_uses_damage_then_stable_tie_break_and_preserves_old_army() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        let tower = app
            .world_mut()
            .query_filtered::<Entity, With<Tower>>()
            .iter(app.world())
            .next()
            .unwrap();
        let old_owner = app.world().get::<Allegiance>(tower).unwrap().0;
        let minion = app
            .world_mut()
            .spawn((Minion, OriginTower(tower), Allegiance(old_owner)))
            .id();
        app.world_mut()
            .resource_mut::<PendingDamage>()
            .0
            .insert(tower, [0.0, 200.0, 200.0, 0.0, 0.0]);
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_damage);
        schedule.run(app.world_mut());
        assert_eq!(app.world().get::<Allegiance>(tower).unwrap().0, factions[1]);
        assert_eq!(app.world().get::<Allegiance>(minion).unwrap().0, old_owner);
        let health = app.world().get::<Health>(tower).unwrap();
        assert_eq!(health.current, health.max);
        assert_eq!(app.world().get::<Spawner>(tower).unwrap().elapsed, 0.0);
    }

    #[test]
    fn ownership_of_all_towers_wins_and_speed_scales_clock() {
        let mut app = app();
        app.world_mut().resource_mut::<MatchState>().speed = 4.0;
        step(&mut app, 1);
        assert!((app.world().resource::<MatchState>().elapsed - 4.0 / 30.0).abs() < 0.0001);
        let faction = app.world().resource::<FactionEntities>().0[2];
        for mut owner in app
            .world_mut()
            .query_filtered::<&mut Allegiance, With<Tower>>()
            .iter_mut(app.world_mut())
        {
            owner.0 = faction;
        }
        step(&mut app, 1);
        let state = app.world().resource::<MatchState>();
        assert_eq!(state.winner, Some(2));
        assert!(state.paused);
    }

    #[test]
    fn spatial_queries_cover_negative_cells_and_ignore_out_of_range_units() {
        let mut index = SpatialIndex::default();
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let location = Vec2::new(-0.1, -8.1);
        index
            .0
            .entry(SpatialIndex::cell(location))
            .or_default()
            .push(Neighbor {
                entity,
                faction: entity,
                position: location,
                tower: false,
            });
        assert_eq!(index.nearby(Vec2::new(0.1, -7.9), 1.0).count(), 1);
        assert_eq!(index.nearby(Vec2::new(2.0, -7.9), 1.0).count(), 0);
    }

    #[test]
    fn movement_stays_inside_battlefield() {
        let mut app = app();
        let factions = app.world().resource::<FactionEntities>().0.clone();
        let unit = app
            .world_mut()
            .spawn((
                Minion,
                Position(Vec2::new(49.9, 49.9)),
                Allegiance(factions[0]),
                Movement {
                    speed: 4.0,
                    target: Vec2::splat(100.0),
                    retarget_in: 999.0,
                },
                Attack {
                    damage: 10.0,
                    range: 1.6,
                    cooldown: 0.8,
                    remaining: 0.0,
                },
            ))
            .id();
        step(&mut app, 30);
        let position = app.world().get::<Position>(unit).unwrap().0;
        assert!(position.is_finite());
        assert!(position.x <= 50.0 && position.y <= 50.0);
    }

    #[test]
    #[ignore = "extended run invoked by scheduled/manual GitHub Actions"]
    fn extended_simulation_remains_bounded() {
        let mut app = app();
        for _ in 0..180 {
            step(&mut app, 100);
            assert!(count::<Minion>(&mut app) <= 500);
            assert_eq!(count::<Tower>(&mut app), 5);
            for (position, health) in app
                .world_mut()
                .query::<(&Position, &Health)>()
                .iter(app.world())
            {
                assert!(position.0.is_finite());
                assert!(position.0.abs().max_element() <= 50.0);
                assert!(health.current > 0.0 && health.current <= health.max);
            }
            let factions = app.world().resource::<FactionEntities>().0.clone();
            for owner in app.world_mut().query::<&Allegiance>().iter(app.world()) {
                assert!(factions.contains(&owner.0));
            }
        }
    }
}
