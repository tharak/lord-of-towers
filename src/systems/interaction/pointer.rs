use crate::components::{
    Allegiance, CombatTarget, DragControl, Health, LegState, Minion, PlayerControlled, Position,
    ShootingPause, Tower, TowerDefenseTarget,
};
use crate::resources::{CameraGesture, InspectorState, MatchState, PlayerSelection, PointerState};
use crate::types::{DragPhase, InputTiming, PointerPress};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub(crate) fn ground_point(
    camera: &Camera,
    transform: &GlobalTransform,
    point: Vec2,
) -> Option<Vec2> {
    let ray = camera.viewport_to_world(transform, point).ok()?;
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    let point = ray.get_point(distance);
    Some(Vec2::new(point.x, point.z))
}
pub(crate) fn select_entity(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    timing: InputTiming,
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    interactions: Query<(&Interaction, &ComputedNode, &UiGlobalTransform)>,
    inspector: Option<Res<InspectorState>>,
    state: Res<MatchState>,
    player: Query<&Allegiance, (With<Tower>, With<PlayerControlled>)>,
    entities: Query<
        (Entity, &Position, &Allegiance, &Health, Option<&Tower>),
        Or<(With<Tower>, With<Minion>)>,
    >,
    mut pointer: ResMut<PointerState>,
    mut gesture: ResMut<CameraGesture>,
    mut selection: ResMut<PlayerSelection>,
) {
    let mut active: Vec<_> = touches.iter().map(|t| (t.id(), t.position())).collect();
    active.sort_by_key(|t| t.0);
    pointer.cooldown = (pointer.cooldown - timing.time.delta_secs()).max(0.0);
    if !active.is_empty()
        || touches.any_just_released()
        || touches.iter_just_canceled().next().is_some()
    {
        pointer.cooldown = 0.75;
    }
    let window = windows.single().ok();
    let inside = |p: Vec2| {
        window.is_some_and(|w| p.x >= 0.0 && p.y >= 0.0 && p.x < w.width() && p.y < w.height())
    };
    let over_ui = |p: Vec2| {
        interactions.iter().any(|(_, node, transform)| {
            Rect::from_center_size(
                transform.translation * node.inverse_scale_factor(),
                node.size() * node.inverse_scale_factor(),
            )
            .contains(p)
        })
    };
    let cancel = |press: Option<PointerPress>, commands: &mut Commands| {
        if let Some(press) = press.filter(|p| p.dragging)
            && let Some(entity) = press.entity
            && let Ok(mut entity) = commands.get_entity(entity)
        {
            entity.remove::<DragControl>().insert(LegState::Idle);
        }
    };
    // Include transient contacts that may have already ended this frame.
    let multiple_touches = active.len() >= 2
        || touches.iter_just_pressed().count() >= 2
        || pointer
            .press
            .and_then(|press| press.touch)
            .is_some_and(|tracked| {
                touches
                    .iter_just_pressed()
                    .any(|touch| touch.id() != tracked)
            });
    if multiple_touches {
        cancel(pointer.press.take(), &mut commands);
        pointer.suppress = true;
    }
    if state.restart_requested
        || state.browser_hidden
        || state.winner.is_some()
        || !window.is_some_and(|w| w.focused)
        || inspector.is_some_and(|s| s.open)
    {
        cancel(pointer.press.take(), &mut commands);
        pointer.previous = None;
        pointer.suppress |= !active.is_empty();
        return;
    }
    if active.is_empty() {
        pointer.suppress = false;
    }
    if multiple_touches && active.len() < 2 {
        pointer.previous = None;
        return;
    }
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    if active.len() >= 2 {
        if active.iter().any(|(_, p)| !inside(*p) || over_ui(*p)) {
            pointer.previous = None;
            return;
        }
        let current = [active[0], active[1]];
        if let Some(previous) = pointer.previous
            && previous[0].0 == current[0].0
            && previous[1].0 == current[1].0
        {
            if let (Some(before), Some(after)) = (
                ground_point(camera, transform, (previous[0].1 + previous[1].1) * 0.5),
                ground_point(camera, transform, (current[0].1 + current[1].1) * 0.5),
            ) {
                gesture.pan += before - after;
            }
            let before = previous[0].1.distance(previous[1].1);
            let after = current[0].1.distance(current[1].1);
            if before > 1.0 && after > 1.0 {
                gesture.zoom *= before / after;
            }
        }
        pointer.previous = Some(current);
        return;
    }
    pointer.previous = None;
    if pointer.suppress {
        return;
    }
    if pointer.press.is_none() {
        let pressed_touch = touches.iter_just_pressed().next();
        let position = pressed_touch.map(|t| t.position()).or_else(|| {
            (pointer.cooldown <= 0.0 && mouse.just_pressed(MouseButton::Left))
                .then(|| window.and_then(Window::cursor_position))
                .flatten()
        });
        let Some(position) = position.filter(|p| inside(*p) && !over_ui(*p)) else {
            return;
        };
        let Some(owner) = player.iter().next() else {
            return;
        };
        let picked = entities
            .iter()
            .filter(|(_, _, allegiance, health, _)| *allegiance == owner && health.current > 0.0)
            .filter_map(|(entity, point, _, _, tower)| {
                let screen = camera
                    .world_to_viewport(
                        transform,
                        Vec3::new(
                            point.0.x,
                            if tower.is_some() { 2.0 } else { 0.5 },
                            point.0.y,
                        ),
                    )
                    .ok()?;
                let distance = screen.distance_squared(position);
                (distance
                    <= if tower.is_some() {
                        38.0_f32.powi(2)
                    } else {
                        28.0_f32.powi(2)
                    })
                .then_some((entity, distance, tower.is_none()))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        // Keep the panel closed during a press so its new bounds cannot intercept the gesture.
        selection.entity = None;
        pointer.press = Some(PointerPress {
            touch: pressed_touch.map(|t| t.id()),
            start: position,
            entity: picked.map(|p| p.0),
            minion: picked.is_some_and(|p| p.2),
            dragging: false,
            moved: false,
        });
    }
    if let Some(mut press) = pointer.press.take() {
        let canceled = press
            .touch
            .is_some_and(|id| touches.iter_just_canceled().any(|t| t.id() == id));
        let released_touch = press
            .touch
            .and_then(|id| touches.iter_just_released().find(|t| t.id() == id));
        let released = released_touch.is_some()
            || (press.touch.is_none() && mouse.just_released(MouseButton::Left));
        let position = if let Some(id) = press.touch {
            released_touch
                .map(|t| t.position())
                .or_else(|| active.iter().find(|t| t.0 == id).map(|t| t.1))
        } else {
            window.and_then(Window::cursor_position)
        };
        let alive = press
            .entity
            .is_none_or(|e| entities.get(e).is_ok_and(|(_, _, _, h, _)| h.current > 0.0));
        if canceled
            || !alive
            || position.is_none_or(|p| !inside(p) || (released && over_ui(p)))
            || (!released && press.touch.is_none() && !mouse.pressed(MouseButton::Left))
        {
            cancel(Some(press), &mut commands);
            return;
        }
        let position = position.unwrap();
        press.moved |= position.distance_squared(press.start) > 64.0;
        if press.minion && !press.dragging && press.moved {
            press.dragging = true;
            selection.entity = None;
            if let Some(entity) = press.entity {
                commands
                    .entity(entity)
                    .insert(ShootingPause(0.0))
                    .remove::<(CombatTarget, TowerDefenseTarget)>();
            }
        }
        if press.dragging {
            selection.entity = None;
            if let Some(target) = ground_point(camera, transform, position) {
                let extent = timing.config.half_extent;
                commands.entity(press.entity.unwrap()).insert((
                    DragControl {
                        target: target.clamp(Vec2::splat(-extent), Vec2::splat(extent)),
                        phase: if released {
                            DragPhase::Released
                        } else {
                            DragPhase::Held
                        },
                    },
                    if released {
                        LegState::DragRelease
                    } else {
                        LegState::DragStart
                    },
                ));
            } else {
                cancel(Some(press), &mut commands);
                return;
            }
        } else if released && !press.moved {
            selection.entity = press.entity;
        }
        if !released {
            pointer.press = Some(press);
        }
        return;
    }
}
