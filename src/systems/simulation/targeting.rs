use crate::components::{
    Allegiance, Attack, CombatTarget, DragControl, ElementInventory, EquipmentOrder, Health,
    InfluenceEmitter, Minion, Movement, Position, SimulationId, Tower, TowerDefenseTarget, Vision,
};
use crate::resources::{PendingTowerAlerts, SpatialIndex};
use bevy::prelude::*;

/// Defense can chase visible attackers; delivery orders otherwise only shoot in range.
pub(crate) fn select_combat_targets(
    mut commands: Commands,
    spatial: Res<SpatialIndex>,
    orders: Query<&EquipmentOrder>,
    towers: Query<(&Health, &Allegiance), With<Tower>>,
    mut minions: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            &Vision,
            &Attack,
            Option<&ElementInventory>,
            Option<&TowerDefenseTarget>,
            Option<&mut Movement>,
            Option<&DragControl>,
        ),
        With<Minion>,
    >,
) {
    for (entity, position, owner, vision, attack, cargo, defense, movement, drag) in &mut minions {
        if drag.is_some() {
            commands.entity(entity).remove::<CombatTarget>();
            continue;
        }
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
        let ordered = orders.iter().any(|order| order.minion == entity);
        let carrying = cargo.is_some_and(|cargo| !cargo.is_empty());
        let target = defense_target.or_else(|| {
            spatial
                .nearby(position.0, vision.radius.max(0.0))
                .filter(|entry| entry.faction != owner.0)
                .filter(|entry| {
                    (!carrying && !ordered)
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
            if ((!carrying && !ordered) || defense_target.is_some())
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
pub(crate) fn process_tower_alerts(
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
        (With<Minion>, Without<DragControl>),
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
