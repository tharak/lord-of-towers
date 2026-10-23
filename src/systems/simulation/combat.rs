use crate::components::{
    Allegiance, Attack, CombatTarget, DragControl, ElementInventory, EquippedElements, Faction,
    Health, Minion, Position, Projectile, ShootingPause, SimulationId, Tower, Vision,
};
use crate::entities::element_deposit::drop_cargo;
use crate::entities::projectile::spawn_projectile;
use crate::resources::{
    MatchState, PendingDamage, PendingTowerAlerts, SimulationConfig, SpatialIndex, VisionOcclusion,
};
use crate::scheduling::simulation_delta;
use crate::types::TowerAlert;
use bevy::prelude::*;

pub(crate) fn clear_pending_damage(mut pending: ResMut<PendingDamage>) {
    pending.0.clear();
}
pub(crate) fn advance_projectiles(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    factions: Query<&Faction>,
    targets: Query<(&Position, &Allegiance, &Health), Without<Projectile>>,
    mut projectiles: Query<(Entity, &mut Position, &Allegiance, &mut Projectile)>,
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
pub(crate) fn collect_damage(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    spatial: Res<SpatialIndex>,
    occlusion: Res<VisionOcclusion>,
    factions: Query<&Faction>,
    live_targets: Query<(&Position, &Allegiance, Option<&Health>), Or<(With<Tower>, With<Minion>)>>,
    mut attackers: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            &mut Attack,
            &Vision,
            Option<&CombatTarget>,
        ),
        (With<Minion>, Without<DragControl>),
    >,
    mut alerts: ResMut<PendingTowerAlerts>,
    config: Res<SimulationConfig>,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, position, faction, mut attack, vision, selected) in &mut attackers {
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
                || !occlusion.has_line_of_sight(position.0, live_position.0)
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
            spawn_projectile(
                &mut commands,
                position.0,
                Allegiance(faction.0),
                Projectile {
                    target: target.entity,
                    damage: attack.damage,
                    speed: config.projectile_speed,
                    remaining: config.projectile_lifetime,
                },
            );
            commands.entity(entity).insert(ShootingPause(1.0));
            attack.remaining = attack.cooldown;
        }
    }
}
pub(crate) fn resolve_damage(
    mut commands: Commands,
    mut pending: ResMut<PendingDamage>,
    mut units: Query<
        (
            &mut Health,
            &Position,
            Option<&ElementInventory>,
            Option<&EquippedElements>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
    ids: Query<&SimulationId>,
) {
    let mut hits: Vec<_> = pending.0.drain().collect();
    hits.sort_by_key(|(entity, _)| ids.get(*entity).map_or(entity.to_bits(), |id| id.0));
    for (entity, contributions) in hits {
        let Ok((mut health, position, cargo, gear)) = units.get_mut(entity) else {
            continue;
        };
        health.current -= contributions.iter().sum::<f32>();
        if health.current > 0.0 {
            continue;
        }
        if let Some(cargo) = cargo
            && !cargo.is_empty()
        {
            drop_cargo(&mut commands, position.0, cargo.clone());
        }
        if let Some(gear) = gear {
            let inventory = ElementInventory {
                units: gear.slots.iter().flatten().copied().collect(),
            };
            if !inventory.is_empty() {
                drop_cargo(&mut commands, position.0, inventory);
            }
        }
        commands.entity(entity).despawn();
    }
}
