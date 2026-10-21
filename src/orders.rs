//! Player destinations are simulation entities; rendering is handled separately.
use crate::{inspector::InspectorState, model::*};
use bevy::{prelude::*, window::PrimaryWindow};

pub struct OrdersPlugin;
impl Plugin for OrdersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttackOrderRequests>()
            .init_resource::<CancelAttackOrderRequests>()
            .add_systems(Update, (gather_pointer_orders, cancel_orders).chain())
            .add_systems(
                FixedUpdate,
                apply_orders
                    .in_set(SimSet::Orders)
                    .after(SimSet::Target)
                    .before(SimSet::Movement),
            );
    }
}

fn gather_pointer_orders(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    touches: Option<Res<Touches>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    interactions: Query<&Interaction>,
    inspector: Option<Res<InspectorState>>,
    state: Res<MatchState>,
    mut requests: ResMut<AttackOrderRequests>,
    mut cancellations: ResMut<CancelAttackOrderRequests>,
) {
    if state.restart_requested
        || state.browser_hidden
        || state.winner.is_some()
        || inspector.is_some_and(|inspector| inspector.open)
        || interactions
            .iter()
            .any(|interaction| *interaction != Interaction::None)
    {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let touch_points: Vec<_> = touches.as_ref().map_or_else(Vec::new, |touches| {
        touches
            .iter_just_pressed()
            .map(|touch| touch.position())
            .collect()
    });
    let mut points = touch_points;
    // Browsers can synthesize mouse events for touches; enqueue each gesture once.
    if points.is_empty()
        && mouse
            .as_ref()
            .is_some_and(|mouse| mouse.just_pressed(MouseButton::Left))
    {
        points.extend(window.cursor_position());
    }
    let cancel_point = mouse
        .as_ref()
        .filter(|mouse| mouse.just_pressed(MouseButton::Right))
        .and_then(|_| window.cursor_position());
    for (point, cancel) in points
        .into_iter()
        .map(|point| (point, false))
        .chain(cancel_point.map(|point| (point, true)))
    {
        let Ok(ray) = camera.viewport_to_world(transform, point) else {
            continue;
        };
        if let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) {
            let point = ray.get_point(distance);
            let point = Vec2::new(point.x, point.z);
            if cancel {
                cancellations.0.push(point);
            } else {
                requests.0.push(point);
            }
        }
    }
}

fn remove_trail(world: &mut World, entity: Entity, assigned: Entity) {
    if world
        .get::<AttackOrder>(assigned)
        .is_some_and(|order| order.trail == entity)
    {
        world.entity_mut(assigned).remove::<AttackOrder>();
    }
    let _ = world.despawn(entity);
}

// Update keeps cancellation and tower-loss cleanup responsive while simulation is paused.
fn cancel_orders(world: &mut World) {
    let points = std::mem::take(&mut world.resource_mut::<CancelAttackOrderRequests>().0);
    let player = world
        .query_filtered::<Entity, (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .next();
    let invalid: Vec<_> = world
        .query::<(Entity, &AttackTrail)>()
        .iter(world)
        .filter(|(_, trail)| Some(trail.tower) != player)
        .map(|(entity, trail)| (entity, trail.assigned))
        .collect();
    for (entity, assigned) in invalid {
        remove_trail(world, entity, assigned);
    }
    if world.resource::<MatchState>().restart_requested {
        return;
    }
    let Some(player) = player else {
        world.resource_mut::<AttackOrderRequests>().0.clear();
        return;
    };
    for point in points.into_iter().filter(|point| point.is_finite()) {
        let nearest = world
            .query::<(Entity, &AttackTrail, &Position)>()
            .iter(world)
            .filter(|(_, trail, _)| trail.tower == player)
            .min_by(|a, b| distance_key(a.2.0, point, a.0, b.2.0, b.0))
            .map(|(entity, trail, _)| (entity, trail.assigned));
        if let Some((entity, assigned)) = nearest {
            remove_trail(world, entity, assigned);
        }
    }
}

fn apply_orders(world: &mut World) {
    if world.resource::<MatchState>().restart_requested {
        world.resource_mut::<AttackOrderRequests>().0.clear();
        return;
    }
    cancel_orders(world);
    if !simulation_active(world.resource::<MatchState>()) {
        return;
    }
    let requests = std::mem::take(&mut world.resource_mut::<AttackOrderRequests>().0);
    for point in requests.into_iter().filter(|point| point.is_finite()) {
        let tower = world
            .query_filtered::<(Entity, &Position, &Allegiance), (With<Tower>, With<PlayerControlled>)>()
            .iter(world)
            .next()
            .map(|(entity, _, allegiance)| (entity, allegiance.0));
        let Some((tower, faction)) = tower else {
            continue;
        };
        let minion = world
            .query_filtered::<(
                Entity,
                &Position,
                &Allegiance,
                Option<&AttackOrder>,
                &OriginTower,
            ), (With<Minion>, With<Movement>)>()
            .iter(world)
            .filter(|(_, _, allegiance, _, origin)| allegiance.0 == faction && origin.0 == tower)
            .min_by(|a, b| {
                a.3.is_some()
                    .cmp(&b.3.is_some())
                    .then_with(|| distance_key(a.1.0, point, a.0, b.1.0, b.0))
            })
            .map(|(entity, _, _, _, _)| entity);
        let Some(minion) = minion else { continue };
        let target = world.query_filtered::<(Entity, &Position, &Allegiance, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>()
            .iter(world)
            .filter(|(_, position, allegiance, tower)| allegiance.0 != faction && position.0.distance(point) <= if tower.is_some() { 2.8 } else { 1.5 })
            .min_by(|a, b| distance_key(a.1.0, point, a.0, b.1.0, b.0))
            .map(|(entity, _, _, _)| entity);
        if let Some(old) = world.get::<AttackOrder>(minion).copied() {
            let _ = world.despawn(old.trail);
        }
        let trail = world
            .spawn((
                Name::new("Player attack trail"),
                MatchEntity,
                EntityKind::AttackTrail,
                Position(point),
                Allegiance(faction),
                AttackTrail {
                    tower,
                    assigned: minion,
                    target,
                },
            ))
            .id();
        world.entity_mut(minion).insert(AttackOrder { trail });
    }
    let trails: Vec<_> = world
        .query::<(Entity, &AttackTrail, &Position, &Allegiance)>()
        .iter(world)
        .map(|(entity, trail, position, allegiance)| (entity, *trail, position.0, allegiance.0))
        .collect();
    for (entity, trail, mut destination, faction) in trails {
        let valid = world.get::<Tower>(trail.tower).is_some()
            && world
                .get::<Allegiance>(trail.tower)
                .is_some_and(|owner| owner.0 == faction)
            && world
                .get::<Allegiance>(trail.assigned)
                .is_some_and(|owner| owner.0 == faction)
            && world
                .get::<AttackOrder>(trail.assigned)
                .is_some_and(|order| order.trail == entity);
        if !valid {
            if world
                .get::<AttackOrder>(trail.assigned)
                .is_some_and(|order| order.trail == entity)
            {
                world.entity_mut(trail.assigned).remove::<AttackOrder>();
            }
            let _ = world.despawn(entity);
            continue;
        }
        if let Some(target) = trail.target {
            if world
                .get::<Allegiance>(target)
                .is_some_and(|owner| owner.0 != faction)
                && let Some(position) = world.get::<Position>(target)
            {
                destination = position.0;
                world.get_mut::<Position>(entity).unwrap().0 = destination;
            } else {
                world.get_mut::<AttackTrail>(entity).unwrap().target = None;
            }
        }
        let movement_target = if let Some(target) = world
            .get::<AttackTrail>(entity)
            .and_then(|trail| trail.target)
        {
            let position = world
                .get::<Position>(trail.assigned)
                .map(|position| position.0);
            let range = world
                .get::<Attack>(trail.assigned)
                .map(|attack| attack.range);
            match (position, range) {
                (Some(position), Some(range))
                    if position.distance(destination)
                        <= range
                            + if world.get::<Tower>(target).is_some() {
                                2.0
                            } else {
                                0.0
                            } =>
                {
                    position
                }
                _ => destination,
            }
        } else {
            destination
        };
        if let Some(mut movement) = world.get_mut::<Movement>(trail.assigned) {
            movement.target = movement_target;
        }
    }
    // Match restart or external cleanup may remove a trail before its minion.
    let stale: Vec<_> = world
        .query::<(Entity, &AttackOrder)>()
        .iter(world)
        .filter(|(_, order)| world.get::<AttackTrail>(order.trail).is_none())
        .map(|(entity, _)| entity)
        .collect();
    for entity in stale {
        world.entity_mut(entity).remove::<AttackOrder>();
    }
}

fn simulation_active(state: &MatchState) -> bool {
    !state.paused && !state.browser_hidden && state.winner.is_none()
}
fn distance_key(
    a: Vec2,
    point: Vec2,
    a_entity: Entity,
    b: Vec2,
    b_entity: Entity,
) -> std::cmp::Ordering {
    a.distance_squared(point)
        .total_cmp(&b.distance_squared(point))
        .then(a_entity.to_bits().cmp(&b_entity.to_bits()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (World, Entity, Entity, Entity) {
        let mut world = World::new();
        world.init_resource::<MatchState>();
        world.init_resource::<AttackOrderRequests>();
        world.init_resource::<CancelAttackOrderRequests>();
        let faction = world.spawn_empty().id();
        let tower = world
            .spawn((
                Tower { slot: 0 },
                PlayerControlled,
                Position(Vec2::ZERO),
                Allegiance(faction),
            ))
            .id();
        let minion = world
            .spawn((
                Minion,
                OriginTower(tower),
                Position(Vec2::ZERO),
                Allegiance(faction),
                Movement {
                    speed: 1.0,
                    target: Vec2::ZERO,
                    retarget_in: 0.0,
                },
            ))
            .id();
        (world, faction, tower, minion)
    }
    #[test]
    fn ground_order_persists_and_reassignment_removes_old_trail() {
        let (mut world, _, _, minion) = setup();
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::X * 8.0);
        apply_orders(&mut world);
        let old = world.get::<AttackOrder>(minion).unwrap().trail;
        world.get_mut::<Movement>(minion).unwrap().target = Vec2::ZERO;
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::X * 8.0);
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::Y * 6.0);
        apply_orders(&mut world);
        assert!(world.get_entity(old).is_err());
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::Y * 6.0);
    }
    #[test]
    fn hostile_target_tracks_then_freezes_and_tower_loss_cleans_up() {
        let (mut world, faction, tower, minion) = setup();
        let enemy = world.spawn_empty().id();
        let target = world
            .spawn((Minion, Position(Vec2::X * 4.0), Allegiance(enemy)))
            .id();
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::X * 4.0);
        apply_orders(&mut world);
        world.get_mut::<Position>(target).unwrap().0 = Vec2::X * 5.0;
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::X * 5.0);
        world.get_mut::<Allegiance>(target).unwrap().0 = faction;
        world.get_mut::<Position>(target).unwrap().0 = Vec2::X * 9.0;
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::X * 5.0);
        world.despawn(tower);
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 0);
    }
    #[test]
    fn orders_only_use_player_origin_even_near_other_towers() {
        let (mut world, faction, player, minion) = setup();
        let other = world
            .spawn((
                Tower { slot: 1 },
                Position(Vec2::X * 100.0),
                Allegiance(faction),
            ))
            .id();
        let foreign = world
            .spawn((
                Minion,
                OriginTower(other),
                Position(Vec2::X * 100.0),
                Allegiance(faction),
                Movement {
                    speed: 1.0,
                    target: Vec2::ZERO,
                    retarget_in: 0.0,
                },
            ))
            .id();
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::X * 100.0);
        apply_orders(&mut world);
        let trail = world.get::<AttackOrder>(minion).unwrap().trail;
        assert_eq!(world.get::<AttackTrail>(trail).unwrap().tower, player);
        assert!(world.get::<AttackOrder>(foreign).is_none());
        world.despawn(player);
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::X * 100.0);
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert!(world.get::<AttackOrder>(foreign).is_none());
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 0);
    }

    #[test]
    fn cancels_nearest_trail_at_any_distance_while_paused() {
        let (mut world, faction, tower, minion) = setup();
        let other_minion = world
            .spawn((
                Minion,
                OriginTower(tower),
                Position(Vec2::X * 20.0),
                Allegiance(faction),
                Movement {
                    speed: 1.0,
                    target: Vec2::ZERO,
                    retarget_in: 0.0,
                },
            ))
            .id();
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::X * 10.0);
        apply_orders(&mut world);
        world
            .resource_mut::<AttackOrderRequests>()
            .0
            .push(Vec2::X * 20.0);
        apply_orders(&mut world);
        let first_trail = world.get::<AttackOrder>(minion).unwrap().trail;
        let nearest_trail = world.get::<AttackOrder>(other_minion).unwrap().trail;
        world.resource_mut::<MatchState>().paused = true;
        world
            .resource_mut::<CancelAttackOrderRequests>()
            .0
            .push(Vec2::X * 10000.0);
        cancel_orders(&mut world);
        assert_eq!(world.get::<AttackOrder>(minion).unwrap().trail, first_trail);
        assert!(world.get::<AttackOrder>(other_minion).is_none());
        assert!(world.get_entity(nearest_trail).is_err());
    }

    #[test]
    fn paused_tower_loss_removes_trails_without_fallback() {
        let (mut world, _, tower, minion) = setup();
        world.resource_mut::<AttackOrderRequests>().0.push(Vec2::X);
        apply_orders(&mut world);
        world.resource_mut::<MatchState>().paused = true;
        world.despawn(tower);
        cancel_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 0);
    }
}
