//! Player destinations are simulation entities; rendering is handled separately.
use crate::{inspector::InspectorState, model::*};
use bevy::{prelude::*, window::PrimaryWindow};

const TRAIL_ERASE_RADIUS: f32 = 2.0;
const RAW_PATH_SPACING: f32 = 0.8;
const MAX_PATH_POINTS: usize = 1024;

#[derive(Clone, Copy)]
enum DrawingPointer {
    Mouse,
    Touch(u64),
}

#[derive(Resource, Default)]
struct DrawingTrail {
    pointer: Option<DrawingPointer>,
    points: Vec<Vec2>,
    touch_navigation: TouchNavigation,
    tool: TrailTool,
}

#[derive(Default)]
struct TouchNavigation {
    suppress_drawing: bool,
    previous: Option<[(u64, Vec2); 2]>,
    mouse_cooldown: f32,
}

impl TouchNavigation {
    fn update(&mut self, touches: &[(u64, Vec2)], touch_event: bool, delta: f32) {
        self.mouse_cooldown = (self.mouse_cooldown - delta).max(0.0);
        if touch_event || !touches.is_empty() {
            self.mouse_cooldown = 0.75;
        }
        if touches.len() >= 2 {
            self.suppress_drawing = true;
        } else {
            self.previous = None;
            if touches.is_empty() {
                self.suppress_drawing = false;
            }
        }
    }
}

// Browser touch taps can leave a synthetic mouse hovering over a control. During
// touch input, hit-test the actual fingers so that hover cannot block the field.
fn ui_blocks_pointer(
    interaction: Interaction,
    touches: &[(u64, Vec2)],
    touch_mouse_cooldown: bool,
    bounds: Rect,
) -> bool {
    if touch_mouse_cooldown {
        bounds.width() > 0.0
            && bounds.height() > 0.0
            && touches
                .iter()
                .any(|(_, position)| bounds.contains(*position))
    } else {
        interaction != Interaction::None
    }
}

fn pinch_ratio(previous: [Vec2; 2], current: [Vec2; 2]) -> Option<f32> {
    let before = previous[0].distance(previous[1]);
    let after = current[0].distance(current[1]);
    (before.is_finite() && after.is_finite() && before > 1.0 && after > 1.0)
        .then_some(before / after)
}

pub struct TrailInputPlugin;
impl Plugin for TrailInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawTrailRequests>()
            .init_resource::<EraseTrailRequests>()
            .init_resource::<TrailIds>()
            .init_resource::<DrawingTrail>()
            .init_resource::<TrailTool>()
            .init_resource::<CameraGesture>()
            .add_systems(Update, (gather_pointer_orders, erase_trails).chain())
            .add_systems(
                FixedUpdate,
                apply_trails
                    .in_set(SimSet::TrailInput)
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
    interactions: Query<(&Interaction, &ComputedNode, &UiGlobalTransform)>,
    inspector: Option<Res<InspectorState>>,
    state: Res<MatchState>,
    tool: Res<TrailTool>,
    mut gesture: ResMut<CameraGesture>,
    time: Option<Res<Time>>,
    mut requests: ResMut<DrawTrailRequests>,
    mut cancellations: ResMut<EraseTrailRequests>,
    mut drawing: ResMut<DrawingTrail>,
) {
    let touches = touches.as_deref();
    let mut active: Vec<_> = touches
        .into_iter()
        .flat_map(|touches| touches.iter())
        .map(|touch| (touch.id(), touch.position()))
        .collect();
    active.sort_by_key(|(id, _)| *id);
    let touch_event = touches.is_some_and(|touches| {
        touches.iter_just_pressed().next().is_some()
            || touches.any_just_released()
            || touches.iter_just_canceled().next().is_some()
    });
    drawing.touch_navigation.update(
        &active,
        touch_event,
        time.as_ref().map_or(1.0 / 60.0, |time| time.delta_secs()),
    );
    let tool_changed = drawing.tool != *tool;
    drawing.tool = *tool;
    if tool_changed {
        drawing.pointer = None;
        drawing.points.clear();
        if !active.is_empty() {
            drawing.touch_navigation.suppress_drawing = true;
        }
    }
    if state.restart_requested
        || state.browser_hidden
        || state.winner.is_some()
        || inspector.is_some_and(|inspector| inspector.open)
        || interactions.iter().any(|(interaction, node, transform)| {
            ui_blocks_pointer(
                *interaction,
                &active,
                drawing.touch_navigation.mouse_cooldown > 0.0,
                Rect::from_center_size(
                    transform.translation * node.inverse_scale_factor(),
                    node.size() * node.inverse_scale_factor(),
                ),
            )
        })
    {
        drawing.pointer = None;
        drawing.points.clear();
        drawing.touch_navigation.previous = None;
        if !active.is_empty() {
            drawing.touch_navigation.suppress_drawing = true;
        }
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let mouse = mouse.as_deref();
    if active.len() >= 2 {
        drawing.pointer = None;
        drawing.points.clear();
        let current = [active[0], active[1]];
        if let Some(previous) = drawing.touch_navigation.previous
            && previous[0].0 == current[0].0
            && previous[1].0 == current[1].0
        {
            let before = [previous[0].1, previous[1].1];
            let after = [current[0].1, current[1].1];
            if let (Some(previous_ground), Some(current_ground)) = (
                ground_point(camera, transform, (before[0] + before[1]) * 0.5),
                ground_point(camera, transform, (after[0] + after[1]) * 0.5),
            ) {
                gesture.pan += previous_ground - current_ground;
            }
            if let Some(ratio) = pinch_ratio(before, after) {
                gesture.zoom *= ratio;
            }
        }
        drawing.touch_navigation.previous = Some(current);
        return;
    }
    if drawing.touch_navigation.suppress_drawing {
        drawing.pointer = None;
        drawing.points.clear();
        return;
    }
    let mouse_allowed = drawing.touch_navigation.mouse_cooldown <= 0.0;
    if !mouse_allowed && matches!(drawing.pointer, Some(DrawingPointer::Mouse)) {
        drawing.pointer = None;
        drawing.points.clear();
    }
    if drawing.pointer.is_none() {
        if let Some(touch) = touches.and_then(|touches| touches.iter_just_pressed().next()) {
            if *tool == TrailTool::Erase {
                if let Some(point) = ground_point(camera, transform, touch.position()) {
                    cancellations.0.push(point);
                }
            } else {
                drawing.pointer = Some(DrawingPointer::Touch(touch.id()));
                if let Some(point) = ground_point(camera, transform, touch.position()) {
                    append_path_point(&mut drawing.points, point);
                }
            }
        } else if mouse_allowed
            && mouse.is_some_and(|mouse| mouse.just_pressed(MouseButton::Left))
            && touches.is_none_or(|touches| !touches.any_just_released())
        {
            if *tool == TrailTool::Draw {
                drawing.pointer = Some(DrawingPointer::Mouse);
            }
            if let Some(point) = window
                .cursor_position()
                .and_then(|position| ground_point(camera, transform, position))
            {
                if *tool == TrailTool::Erase {
                    cancellations.0.push(point);
                } else {
                    append_path_point(&mut drawing.points, point);
                }
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
    if mouse_allowed
        && mouse.is_some_and(|mouse| mouse.just_pressed(MouseButton::Right))
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
    if !point.is_finite() || points.len() >= MAX_PATH_POINTS {
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
    let steps = (distance / RAW_PATH_SPACING).ceil() as usize;
    for step in 1..=steps.min(MAX_PATH_POINTS - points.len()) {
        points.push(start.lerp(point, step as f32 / steps as f32));
    }
}

// Erasing remains responsive while paused. Only player markers participate.
fn erase_trails(world: &mut World) {
    let points = std::mem::take(&mut world.resource_mut::<EraseTrailRequests>().0);
    if world.resource::<MatchState>().restart_requested {
        world.resource_mut::<DrawTrailRequests>().0.clear();
        return;
    }
    for point in points.into_iter().filter(|p| p.is_finite()) {
        let markers: Vec<_> = world
            .query_filtered::<(Entity, &Trail, &Position), With<PlayerDrawnTrail>>()
            .iter(world)
            .map(|(e, t, p)| (e, *t, p.0))
            .collect();
        let nearest = markers
            .iter()
            .map(|(entity, trail, position)| {
                // Consecutive surviving markers define a segment even after budget eviction.
                let previous = markers
                    .iter()
                    .filter(|(_, other, _)| {
                        other.trip == trail.trip && other.sequence > trail.sequence
                    })
                    .min_by_key(|(_, other, _)| other.sequence);
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
                (*entity, trail.trip, distance)
            })
            .filter(|(_, _, d)| *d <= TRAIL_ERASE_RADIUS.powi(2))
            .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.to_bits().cmp(&b.0.to_bits())));
        if let Some((_, trip, _)) = nearest {
            for (entity, trail, _) in markers {
                if trail.trip == trip {
                    world.despawn(entity);
                }
            }
        }
    }
    crate::gathering::clean_trail_memory(world);
}

fn accepted_points(points: Vec<Vec2>, spacing: f32) -> Vec<Vec2> {
    let mut accepted: Vec<Vec2> = Vec::new();
    for point in points
        .into_iter()
        .take(MAX_PATH_POINTS)
        .filter(|p| p.is_finite())
    {
        if accepted
            .iter()
            .all(|previous| previous.distance(point) >= spacing)
        {
            accepted.push(point);
        }
    }
    accepted
}

fn apply_trails(world: &mut World) {
    erase_trails(world);
    if world.resource::<MatchState>().restart_requested {
        return;
    }
    let player = world
        .query_filtered::<&Allegiance, (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .next()
        .copied();
    let Some(owner) = player else {
        world.resource_mut::<DrawTrailRequests>().0.clear();
        return;
    };
    let state = world.resource::<MatchState>();
    if state.paused || state.browser_hidden || state.winner.is_some() {
        return;
    }
    let spacing = world.resource::<SimulationConfig>().vision_radius * 0.5;
    let capacity = world.resource::<SimulationConfig>().drawn_trail_capacity;
    let requests = std::mem::take(&mut world.resource_mut::<DrawTrailRequests>().0);
    for points in requests {
        let points = accepted_points(points, spacing);
        if points.is_empty() || capacity == 0 {
            continue;
        }
        let trip = crate::gathering::allocate_trip(world);
        let count = points.len();
        for (index, point) in points.into_iter().enumerate() {
            while world.query::<&PlayerDrawnTrail>().iter(world).count() >= capacity {
                let oldest = world
                    .query::<(Entity, &PlayerDrawnTrail)>()
                    .iter(world)
                    .min_by_key(|(e, t)| (t.serial, e.to_bits()))
                    .map(|(e, _)| e)
                    .unwrap();
                world.despawn(oldest);
            }
            let serial = {
                let mut ids = world.resource_mut::<TrailIds>();
                let serial = ids.next_player_marker;
                ids.next_player_marker += 1;
                serial
            };
            let entity =
                crate::gathering::marker(world, point, owner, trip, (count - index - 1) as u32);
            world.entity_mut(entity).insert(PlayerDrawnTrail { serial });
        }
    }
    crate::gathering::clean_trail_memory(world);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> World {
        let mut w = World::new();
        w.init_resource::<MatchState>();
        w.init_resource::<SimulationConfig>();
        w.init_resource::<DrawTrailRequests>();
        w.init_resource::<EraseTrailRequests>();
        w.init_resource::<TrailIds>();
        let faction = w.spawn_empty().id();
        w.spawn((Tower { slot: 0 }, PlayerControlled, Allegiance(faction)));
        w
    }
    fn draw(w: &mut World, points: Vec<Vec2>) {
        w.resource_mut::<DrawTrailRequests>().0.push(points);
        apply_trails(w);
    }
    #[test]
    fn spacing_checks_entire_stroke_and_does_not_force_endpoint() {
        assert_eq!(
            accepted_points(vec![Vec2::ZERO, Vec2::X * 5.0, Vec2::Y, Vec2::X * 5.1], 5.0),
            vec![Vec2::ZERO, Vec2::X * 5.0]
        );
        assert_eq!(
            accepted_points(vec![Vec2::NAN, Vec2::ZERO], 5.0),
            vec![Vec2::ZERO]
        );
    }
    #[test]
    fn global_budget_evicts_oldest_only_and_sequences_point_toward_end() {
        let mut w = world();
        w.resource_mut::<SimulationConfig>().drawn_trail_capacity = 3;
        let owner = Allegiance(w.spawn_empty().id());
        let carrier_trip = crate::gathering::allocate_trip(&mut w);
        let carrier = crate::gathering::marker(&mut w, Vec2::ZERO, owner, carrier_trip, 0);
        draw(&mut w, vec![Vec2::ZERO, Vec2::X * 20.0, Vec2::X * 40.0]);
        let trip = w
            .query_filtered::<&Trail, With<PlayerDrawnTrail>>()
            .iter(&w)
            .next()
            .unwrap()
            .trip;
        assert_ne!(trip, carrier_trip);
        let mut sequence: Vec<_> = w
            .query_filtered::<(&Trail, &Position), With<PlayerDrawnTrail>>()
            .iter(&w)
            .map(|(t, p)| (p.0.x, t.sequence))
            .collect();
        sequence.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert_eq!(sequence, vec![(0.0, 2), (20.0, 1), (40.0, 0)]);
        draw(&mut w, vec![Vec2::Y * 20.0]);
        assert_eq!(w.query::<&PlayerDrawnTrail>().iter(&w).count(), 3);
        assert!(w.get::<Trail>(carrier).is_some());
        assert!(
            w.query_filtered::<&Position, With<PlayerDrawnTrail>>()
                .iter(&w)
                .all(|p| p.0 != Vec2::ZERO)
        );
        assert!(w.query::<&Trail>().iter(&w).all(|t| t.remaining == 30.0));
    }
    #[test]
    fn segment_erasing_handles_sequence_gaps_and_preserves_carriers() {
        let mut w = world();
        draw(&mut w, vec![Vec2::ZERO, Vec2::X * 20.0, Vec2::X * 40.0]);
        let middle = w
            .query_filtered::<(Entity, &Trail), With<PlayerDrawnTrail>>()
            .iter(&w)
            .find(|(_, t)| t.sequence == 1)
            .unwrap()
            .0;
        w.despawn(middle);
        let owner = Allegiance(w.spawn_empty().id());
        let trip = crate::gathering::allocate_trip(&mut w);
        let carrier = crate::gathering::marker(&mut w, Vec2::X * 20.0, owner, trip, 0);
        w.resource_mut::<MatchState>().paused = true;
        w.resource_mut::<EraseTrailRequests>()
            .0
            .push(Vec2::X * 20.0 + Vec2::Y * 1.9);
        erase_trails(&mut w);
        assert_eq!(w.query::<&PlayerDrawnTrail>().iter(&w).count(), 0);
        assert!(w.get::<Trail>(carrier).is_some());
    }
    #[test]
    fn tower_loss_preserves_markers_but_blocks_new_strokes() {
        let mut w = world();
        draw(&mut w, vec![Vec2::ZERO]);
        let tower = w
            .query_filtered::<Entity, With<Tower>>()
            .iter(&w)
            .next()
            .unwrap();
        w.despawn(tower);
        draw(&mut w, vec![Vec2::X * 20.0]);
        assert_eq!(w.query::<&Trail>().iter(&w).count(), 1);
        assert!(w.resource::<DrawTrailRequests>().0.is_empty());
    }
    #[test]
    fn touch_stroke_ignores_synthetic_ui_hover_but_blocks_actual_ui_touches() {
        let bounds = Rect::from_center_size(Vec2::new(195.0, 70.0), Vec2::new(80.0, 40.0));
        let field_touch = [(0, Vec2::new(150.0, 420.0))];
        assert!(!ui_blocks_pointer(
            Interaction::Hovered,
            &field_touch,
            true,
            bounds
        ));
        // A released field stroke must still finish while the mouse hovers UI.
        assert!(!ui_blocks_pointer(Interaction::Hovered, &[], true, bounds));
        let control_touch = [(0, bounds.center())];
        assert!(ui_blocks_pointer(
            Interaction::None,
            &control_touch,
            true,
            bounds
        ));
        assert!(ui_blocks_pointer(Interaction::Hovered, &[], false, bounds));
        assert!(!ui_blocks_pointer(Interaction::None, &[], false, bounds));
    }

    #[test]
    fn two_finger_navigation_latches_until_all_fingers_lift() {
        let mut n = TouchNavigation::default();
        n.update(&[(1, Vec2::ZERO), (2, Vec2::X)], true, 0.016);
        n.update(&[(1, Vec2::ZERO)], true, 0.016);
        assert!(n.suppress_drawing);
        n.update(&[], true, 0.016);
        assert!(!n.suppress_drawing);
        assert!(n.mouse_cooldown > 0.0);
        assert_eq!(
            pinch_ratio([Vec2::ZERO, Vec2::X * 20.0], [Vec2::ZERO, Vec2::X * 40.0]),
            Some(0.5)
        );
    }
}
