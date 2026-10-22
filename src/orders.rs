//! Player destinations are simulation entities; rendering is handled separately.
use crate::{inspector::InspectorState, model::*};
use bevy::{prelude::*, window::PrimaryWindow};

const ATTACK_TRAIL_CANCEL_RADIUS: f32 = 2.0;
const ATTACK_PATH_SPACING: f32 = 0.8;
const MAX_ATTACK_PATH_POINTS: usize = 1024;

#[derive(Clone, Copy)]
enum DrawingPointer {
    Mouse,
    Touch(u64),
}

#[derive(Resource, Default)]
struct DrawingAttackPath {
    pointer: Option<DrawingPointer>,
    points: Vec<Vec2>,
}

pub struct OrdersPlugin;
impl Plugin for OrdersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttackOrderRequests>()
            .init_resource::<CancelAttackOrderRequests>()
            .init_resource::<AttackPathIds>()
            .init_resource::<DrawingAttackPath>()
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
    mut drawing: ResMut<DrawingAttackPath>,
) {
    if state.restart_requested
        || state.browser_hidden
        || state.winner.is_some()
        || inspector.is_some_and(|inspector| inspector.open)
        || interactions
            .iter()
            .any(|interaction| *interaction != Interaction::None)
    {
        drawing.pointer = None;
        drawing.points.clear();
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let mouse = mouse.as_deref();
    let touches = touches.as_deref();
    if drawing.pointer.is_none() {
        if let Some(touch) = touches.and_then(|touches| touches.iter_just_pressed().next()) {
            drawing.pointer = Some(DrawingPointer::Touch(touch.id()));
            if let Some(point) = ground_point(camera, transform, touch.position()) {
                append_path_point(&mut drawing.points, point);
            }
        } else if mouse.is_some_and(|mouse| mouse.just_pressed(MouseButton::Left))
            && touches.is_none_or(|touches| !touches.any_just_released())
        {
            drawing.pointer = Some(DrawingPointer::Mouse);
            if let Some(point) = window
                .cursor_position()
                .and_then(|position| ground_point(camera, transform, position))
            {
                append_path_point(&mut drawing.points, point);
            }
        }
    }
    let mut finished = false;
    match drawing.pointer {
        Some(DrawingPointer::Mouse) => {
            if let Some(point) = window
                .cursor_position()
                .and_then(|position| ground_point(camera, transform, position))
            {
                append_path_point(&mut drawing.points, point);
            }
            finished = mouse.is_none_or(|mouse| mouse.just_released(MouseButton::Left));
        }
        Some(DrawingPointer::Touch(id)) => {
            if let Some(touch) = touches.and_then(|touches| touches.get_pressed(id))
                && let Some(point) = ground_point(camera, transform, touch.position())
            {
                append_path_point(&mut drawing.points, point);
            }
            if let Some(touch) = touches.and_then(|touches| touches.get_released(id)) {
                if let Some(point) = ground_point(camera, transform, touch.position()) {
                    append_path_point(&mut drawing.points, point);
                }
                finished = true;
            } else if touches.is_none_or(|touches| touches.just_canceled(id)) {
                drawing.pointer = None;
                drawing.points.clear();
            }
        }
        None => {}
    }
    if finished {
        drawing.pointer = None;
        if !drawing.points.is_empty() {
            requests.0.push(std::mem::take(&mut drawing.points));
        }
    }
    if mouse.is_some_and(|mouse| mouse.just_pressed(MouseButton::Right))
        && let Some(point) = window
            .cursor_position()
            .and_then(|position| ground_point(camera, transform, position))
    {
        cancellations.0.push(point);
    }
}

fn ground_point(camera: &Camera, transform: &GlobalTransform, point: Vec2) -> Option<Vec2> {
    let ray = camera.viewport_to_world(transform, point).ok()?;
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    let point = ray.get_point(distance);
    let ground = Vec2::new(point.x, point.z);
    ground.is_finite().then_some(ground)
}

fn append_path_point(points: &mut Vec<Vec2>, point: Vec2) {
    if !point.is_finite() || points.len() >= MAX_ATTACK_PATH_POINTS {
        return;
    }
    let Some(start) = points.last().copied() else {
        points.push(point);
        return;
    };
    let distance = start.distance(point);
    if distance < 0.001 {
        return;
    }
    let steps = (distance / ATTACK_PATH_SPACING).ceil() as usize;
    for step in 1..=steps.min(MAX_ATTACK_PATH_POINTS - points.len()) {
        points.push(start.lerp(point, step as f32 / steps as f32));
    }
}

fn remove_trail(world: &mut World, entity: Entity) {
    let followers: Vec<_> = world
        .query::<(Entity, &AttackOrder)>()
        .iter(world)
        .filter(|(_, order)| order.trail == entity)
        .map(|(entity, _)| entity)
        .collect();
    for follower in followers {
        world.entity_mut(follower).remove::<AttackOrder>();
    }
    let _ = world.despawn(entity);
}

// Update keeps cancellation and tower-loss cleanup responsive while simulation is paused.
fn cancel_orders(world: &mut World) {
    // Release cargo carriers before gathering navigation runs, including while paused.
    let carriers: Vec<_> = world
        .query::<(Entity, &AttackOrder, &RawElement)>()
        .iter(world)
        .filter(|(_, _, cargo)| cargo.0 > 0)
        .map(|(entity, _, _)| entity)
        .collect();
    for entity in carriers {
        world.entity_mut(entity).remove::<AttackOrder>();
    }
    let points = std::mem::take(&mut world.resource_mut::<CancelAttackOrderRequests>().0);
    let player = world
        .query_filtered::<Entity, (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .next();
    let invalid: Vec<_> = world
        .query::<(Entity, &AttackTrail)>()
        .iter(world)
        .filter(|(_, trail)| {
            world.resource::<MatchState>().restart_requested || Some(trail.tower) != player
        })
        .map(|(entity, _)| entity)
        .collect();
    for entity in invalid {
        remove_trail(world, entity);
    }
    if world.resource::<MatchState>().restart_requested {
        return;
    }
    let Some(player) = player else {
        world.resource_mut::<AttackOrderRequests>().0.clear();
        return;
    };
    for point in points.into_iter().filter(|point| point.is_finite()) {
        let markers: Vec<_> = world
            .query::<(Entity, &AttackTrail, &Position)>()
            .iter(world)
            .filter(|(_, trail, _)| trail.tower == player)
            .map(|(entity, trail, position)| (entity, *trail, position.0))
            .collect();
        let nearest = markers
            .iter()
            .map(|(entity, trail, position)| {
                let previous = markers.iter().find(|(_, other, _)| {
                    other.path_id == trail.path_id
                        && other.sequence.checked_add(1) == Some(trail.sequence)
                });
                let distance = previous.map_or_else(
                    || position.distance_squared(point),
                    |(_, _, start)| {
                        let offset = *position - *start;
                        let t = if offset.length_squared() > 0.0 {
                            ((point - *start).dot(offset) / offset.length_squared()).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        (*start + offset * t).distance_squared(point)
                    },
                );
                (*entity, trail.path_id, distance)
            })
            .filter(|(_, _, distance)| *distance <= ATTACK_TRAIL_CANCEL_RADIUS.powi(2))
            .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.to_bits().cmp(&b.0.to_bits())));
        if let Some((_, path, _)) = nearest {
            for (entity, trail, _) in markers {
                if trail.path_id == path {
                    remove_trail(world, entity);
                }
            }
        }
    }
}

fn apply_orders(world: &mut World) {
    cancel_orders(world);
    if world.resource::<MatchState>().restart_requested {
        world.resource_mut::<AttackOrderRequests>().0.clear();
        return;
    }
    if !simulation_active(world.resource::<MatchState>()) {
        return;
    }
    // Signals placed this step become discoverable on the next simulation step.
    let discoverable: Vec<_> = world
        .query_filtered::<Entity, With<AttackTrail>>()
        .iter(world)
        .collect();
    let requests = std::mem::take(&mut world.resource_mut::<AttackOrderRequests>().0);
    for points in requests {
        let points: Vec<_> = points
            .into_iter()
            .filter(|point| point.is_finite())
            .collect();
        if points.is_empty() {
            continue;
        }
        let tower = world
            .query_filtered::<(Entity, &Position, &Allegiance), (With<Tower>, With<PlayerControlled>)>()
            .iter(world)
            .next()
            .map(|(entity, _, allegiance)| (entity, allegiance.0));
        let Some((tower, faction)) = tower else {
            continue;
        };
        world.resource_mut::<AttackPathIds>().0 += 1;
        let path_id = world.resource::<AttackPathIds>().0;
        let last = points.len() - 1;
        for (sequence, point) in points.into_iter().enumerate() {
            let target = world.query_filtered::<(Entity, &Position, &Allegiance, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>()
            .iter(world)
            .filter(|(_, position, allegiance, tower)| allegiance.0 != faction && position.0.distance(point) <= if tower.is_some() { 2.8 } else { 1.5 })
            .min_by(|a, b| distance_key(a.1.0, point, a.0, b.1.0, b.0))
            .map(|(entity, _, _, _)| entity);
            world.spawn((
                Name::new("Player attack trail"),
                MatchEntity,
                EntityKind::AttackTrail,
                Position(point),
                Allegiance(faction),
                AttackTrail {
                    tower,
                    path_id,
                    sequence: sequence as u32,
                    target: if sequence == last { target } else { None },
                },
            ));
        }
    }
    let trails: Vec<_> = world
        .query::<(Entity, &AttackTrail, &Position, &Allegiance)>()
        .iter(world)
        .map(|(entity, trail, position, allegiance)| (entity, *trail, position.0, allegiance.0))
        .collect();
    for (entity, trail, _, faction) in trails {
        let valid = world.get::<Tower>(trail.tower).is_some()
            && world
                .get::<Allegiance>(trail.tower)
                .is_some_and(|owner| owner.0 == faction);
        if !valid {
            remove_trail(world, entity);
            continue;
        }
        if let Some(target) = trail.target {
            if world
                .get::<Allegiance>(target)
                .is_some_and(|owner| owner.0 != faction)
                && let Some(position) = world.get::<Position>(target)
            {
                let destination = position.0;
                world.get_mut::<Position>(entity).unwrap().0 = destination;
            } else {
                world.get_mut::<AttackTrail>(entity).unwrap().target = None;
            }
        }
    }
    let minions: Vec<_> = world.query_filtered::<(Entity, &Position, &Allegiance, &OriginTower, &Vision), (With<Minion>, With<Movement>)>()
        .iter(world).map(|(entity, position, owner, origin, vision)| (entity, position.0, owner.0, origin.0, vision.radius)).collect();
    for (minion, position, faction, origin, radius) in minions {
        if world
            .get::<RawElement>(minion)
            .is_some_and(|cargo| cargo.0 > 0)
        {
            world.entity_mut(minion).remove::<AttackOrder>();
            continue;
        }
        let eligible =
            |trail: &AttackTrail, owner: &Allegiance| trail.tower == origin && owner.0 == faction;
        let current = world.get::<AttackOrder>(minion).and_then(|order| {
            let trail = world.get::<AttackTrail>(order.trail)?;
            let owner = world.get::<Allegiance>(order.trail)?;
            eligible(trail, owner).then_some(order.trail)
        });
        let chosen = current.or_else(|| {
            world
                .query::<(Entity, &AttackTrail, &Position, &Allegiance)>()
                .iter(world)
                .filter(|(entity, trail, destination, owner)| {
                    discoverable.contains(entity)
                        && eligible(trail, owner)
                        && position.distance_squared(destination.0) <= radius.max(0.0).powi(2)
                })
                .min_by(|a, b| distance_key(a.2.0, position, a.0, b.2.0, b.0))
                .map(|(entity, _, _, _)| entity)
        });
        let Some(mut trail) = chosen else {
            world.entity_mut(minion).remove::<AttackOrder>();
            continue;
        };
        loop {
            let marker = *world.get::<AttackTrail>(trail).unwrap();
            if position.distance(world.get::<Position>(trail).unwrap().0) > 1.0 {
                break;
            }
            let next = world
                .query::<(Entity, &AttackTrail)>()
                .iter(world)
                .filter(|(_, next)| {
                    next.path_id == marker.path_id && next.sequence > marker.sequence
                })
                .min_by_key(|(entity, next)| (next.sequence, entity.to_bits()))
                .map(|(entity, _)| entity);
            let Some(next) = next else {
                break;
            };
            trail = next;
        }
        let destination = world.get::<Position>(trail).unwrap().0;
        let target = world.get::<AttackTrail>(trail).unwrap().target;
        let movement_target = if target.is_some_and(|target| {
            world.get::<Attack>(minion).is_some_and(|attack| {
                position.distance(destination)
                    <= attack.range
                        + if world.get::<Tower>(target).is_some() {
                            2.0
                        } else {
                            0.0
                        }
            })
        }) {
            position
        } else {
            destination
        };
        world.entity_mut(minion).insert(AttackOrder { trail });
        world.get_mut::<Movement>(minion).unwrap().target = movement_target;
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
    #[test]
    fn drawn_path_samples_long_strokes_and_keeps_endpoints() {
        let mut points = Vec::new();
        append_path_point(&mut points, Vec2::ZERO);
        append_path_point(&mut points, Vec2::X * 5.0);
        assert_eq!(points.first(), Some(&Vec2::ZERO));
        assert_eq!(points.last(), Some(&(Vec2::X * 5.0)));
        assert!(points.len() > 2);
        assert!(
            points
                .windows(2)
                .all(|pair| pair[0].distance(pair[1]) <= ATTACK_PATH_SPACING + 0.001)
        );
    }
    fn setup() -> (World, Entity, Entity, Entity) {
        let mut world = World::new();
        world.init_resource::<MatchState>();
        world.init_resource::<AttackOrderRequests>();
        world.init_resource::<CancelAttackOrderRequests>();
        world.init_resource::<AttackPathIds>();
        let faction = world.spawn_empty().id();
        let tower = world
            .spawn((
                Tower { slot: 0 },
                PlayerControlled,
                Position(Vec2::ZERO),
                Allegiance(faction),
            ))
            .id();
        let minion = unit(&mut world, faction, tower, Vec2::ZERO);
        (world, faction, tower, minion)
    }
    fn unit(world: &mut World, faction: Entity, tower: Entity, point: Vec2) -> Entity {
        world
            .spawn((
                Minion,
                OriginTower(tower),
                Position(point),
                Allegiance(faction),
                Vision { radius: 5.0 },
                RawElement(0),
                Movement {
                    speed: 1.0,
                    target: Vec2::ZERO,
                    retarget_in: 0.0,
                },
            ))
            .id()
    }
    fn draw(world: &mut World, points: Vec<Vec2>) {
        world.resource_mut::<AttackOrderRequests>().0.push(points);
        apply_orders(world);
    }
    #[test]
    fn placement_selects_no_minion_and_multiple_units_discover_in_vision() {
        let (mut world, faction, tower, minion) = setup();
        let second = unit(&mut world, faction, tower, Vec2::X);
        let far = unit(&mut world, faction, tower, Vec2::X * 30.0);
        let foreign_tower = world.spawn_empty().id();
        let foreign = unit(&mut world, faction, foreign_tower, Vec2::ZERO);
        draw(&mut world, vec![Vec2::X * 4.0]);
        assert!(world.get::<AttackOrder>(minion).is_none());
        apply_orders(&mut world);
        assert_eq!(
            world.get::<AttackOrder>(minion).unwrap().trail,
            world.get::<AttackOrder>(second).unwrap().trail
        );
        assert!(world.get::<AttackOrder>(far).is_none());
        assert!(world.get::<AttackOrder>(foreign).is_none());
        world.get_mut::<Position>(far).unwrap().0 = Vec2::X * 7.0;
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(far).is_some());
    }
    #[test]
    fn follows_ordered_waypoints_and_holds_final_destination() {
        let (mut world, _, _, minion) = setup();
        draw(
            &mut world,
            vec![Vec2::X * 2.0, Vec2::Y * 8.0, Vec2::X * 12.0],
        );
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::X * 2.0);
        world.get_mut::<Position>(minion).unwrap().0 = Vec2::X * 2.0;
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::Y * 8.0);
        world.get_mut::<Position>(minion).unwrap().0 = Vec2::Y * 8.0;
        apply_orders(&mut world);
        assert_eq!(
            world.get::<Movement>(minion).unwrap().target,
            Vec2::X * 12.0
        );
        world.get_mut::<Position>(minion).unwrap().0 = Vec2::X * 12.0;
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_some());
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 3);
    }
    #[test]
    fn cargo_is_never_diverted_and_releases_existing_following() {
        let (mut world, _, _, minion) = setup();
        world.get_mut::<RawElement>(minion).unwrap().0 = 1;
        world.get_mut::<Movement>(minion).unwrap().target = Vec2::Y;
        draw(&mut world, vec![Vec2::X * 2.0]);
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::Y);
        world.get_mut::<RawElement>(minion).unwrap().0 = 0;
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_some());
        world.get_mut::<RawElement>(minion).unwrap().0 = 1;
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
    }
    #[test]
    fn cancellation_near_segment_removes_whole_path_and_all_followers_while_paused() {
        let (mut world, faction, tower, minion) = setup();
        let second = unit(&mut world, faction, tower, Vec2::ZERO);
        draw(&mut world, vec![Vec2::X * 2.0, Vec2::X * 12.0]);
        apply_orders(&mut world);
        draw(&mut world, vec![Vec2::Y * 20.0]);
        world.resource_mut::<MatchState>().paused = true;
        world
            .resource_mut::<CancelAttackOrderRequests>()
            .0
            .push(Vec2::splat(100.0));
        cancel_orders(&mut world);
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 3);
        world
            .resource_mut::<CancelAttackOrderRequests>()
            .0
            .push(Vec2::X * 7.0 + Vec2::Y * 1.9);
        cancel_orders(&mut world);
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 1);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert!(world.get::<AttackOrder>(second).is_none());
    }
    #[test]
    fn final_hostile_tracks_then_freezes_and_tower_loss_cleans_up() {
        let (mut world, faction, tower, minion) = setup();
        let enemy = world.spawn_empty().id();
        let target = world
            .spawn((Minion, Position(Vec2::X * 4.0), Allegiance(enemy)))
            .id();
        draw(&mut world, vec![Vec2::X * 4.0]);
        apply_orders(&mut world);
        world.get_mut::<Position>(target).unwrap().0 = Vec2::X * 9.0;
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::X * 9.0);
        world.get_mut::<Allegiance>(target).unwrap().0 = faction;
        world.get_mut::<Position>(target).unwrap().0 = Vec2::X * 20.0;
        apply_orders(&mut world);
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::X * 9.0);
        world.resource_mut::<MatchState>().paused = true;
        world.despawn(tower);
        cancel_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 0);
    }
    #[test]
    fn hostile_endpoint_holds_follower_within_attack_range() {
        let (mut world, _, _, minion) = setup();
        let enemy = world.spawn_empty().id();
        world.spawn((Minion, Position(Vec2::X * 4.0), Allegiance(enemy)));
        world.entity_mut(minion).insert(Attack {
            damage: 1.0,
            range: 5.0,
            cooldown: 1.0,
            remaining: 0.0,
        });
        draw(&mut world, vec![Vec2::X * 4.0]);
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_some());
        assert_eq!(world.get::<Movement>(minion).unwrap().target, Vec2::ZERO);
    }
    #[test]
    fn restart_removes_paths_and_followers() {
        let (mut world, _, _, minion) = setup();
        draw(&mut world, vec![Vec2::X]);
        apply_orders(&mut world);
        world.resource_mut::<MatchState>().restart_requested = true;
        apply_orders(&mut world);
        assert!(world.get::<AttackOrder>(minion).is_none());
        assert_eq!(world.query::<&AttackTrail>().iter(&world).count(), 0);
    }
}
