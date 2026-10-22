//! Friendly unit selection and contextual equipment/production controls.
use crate::{inspector::InspectorState, model::*};
use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
struct PointerState {
    previous: Option<[(u64, Vec2); 2]>,
    suppress: bool,
    cooldown: f32,
    press: Option<PointerPress>,
}
#[derive(Clone, Copy)]
struct PointerPress {
    touch: Option<u64>,
    start: Vec2,
    entity: Option<Entity>,
    minion: bool,
    dragging: bool,
    moved: bool,
}
#[derive(SystemParam)]
struct InputTiming<'w> {
    time: Res<'w, Time>,
    config: Res<'w, SimulationConfig>,
}
#[derive(Component)]
struct ContextPanel;
#[derive(Component, Clone, Debug)]
enum ContextAction {
    Equipment {
        command: PlayerCommand,
        cancel: bool,
    },
    Recipe {
        tower: Entity,
        element: Element,
    },
}
#[derive(Resource, Default)]
struct PanelCache(String);

pub struct PlayerInteractionPlugin;
impl Plugin for PlayerInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSelection>()
            .init_resource::<PlayerCommands>()
            .init_resource::<CameraGesture>()
            .init_resource::<PointerState>()
            .init_resource::<PanelCache>()
            .add_systems(
                Update,
                (select_entity, context_actions, context_panel).chain(),
            );
    }
}

fn ground_point(camera: &Camera, transform: &GlobalTransform, point: Vec2) -> Option<Vec2> {
    let ray = camera.viewport_to_world(transform, point).ok()?;
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    let point = ray.get_point(distance);
    Some(Vec2::new(point.x, point.z))
}

fn select_entity(
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
    if active.len() >= 2 {
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
            || position.is_none_or(|p| !inside(p) || over_ui(p))
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

fn context_actions(
    actions: Query<(&Interaction, &ContextAction), Changed<Interaction>>,
    mut selection: ResMut<PlayerSelection>,
    mut commands: ResMut<PlayerCommands>,
    state: Res<MatchState>,
    towers: Query<(&ElementInventory, &Health), With<Tower>>,
) {
    if state.restart_requested || state.browser_hidden || state.winner.is_some() {
        return;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            ContextAction::Equipment { command, cancel } => {
                if *cancel {
                    if let PlayerCommand::Equip { minion, .. }
                    | PlayerCommand::ReturnEquipment { minion, .. } = command
                    {
                        commands
                            .0
                            .push(PlayerCommand::CancelOrder { minion: *minion });
                    }
                }
                commands.0.push(command.clone());
            }
            ContextAction::Recipe { tower, element } => {
                if selection.entity != Some(*tower) {
                    continue;
                }
                let Ok((inventory, health)) = towers.get(*tower) else {
                    continue;
                };
                if health.current <= 0.0 {
                    continue;
                }
                let stock = recipe_stock(inventory, *tower, &commands);
                for (selected, available) in selection.recipe.iter_mut().zip(stock) {
                    *selected = (*selected).min(available);
                }
                let index = *element as usize;
                if selection.recipe[index] >= stock[index] {
                    continue;
                }
                selection.recipe[index] += 1;
                if selection.recipe.iter().sum::<u32>() == ELEMENTS_PER_MINION as u32 {
                    commands.0.push(PlayerCommand::CreateMinion {
                        tower: *tower,
                        counts: selection.recipe,
                    });
                    selection.recipe = [0; 5];
                }
            }
        }
    }
}

struct Row {
    label: String,
    action: Option<ContextAction>,
    color: Color,
}
fn row(label: impl Into<String>, action: Option<ContextAction>) -> Row {
    Row {
        label: label.into(),
        action,
        color: Color::srgb(0.15, 0.19, 0.25),
    }
}

// Queued births also claim stock while simulation is paused or between fixed ticks.
fn recipe_stock(
    inventory: &ElementInventory,
    tower: Entity,
    commands: &PlayerCommands,
) -> [u32; 5] {
    let mut stock = inventory.counts();
    for command in &commands.0 {
        if let PlayerCommand::CreateMinion {
            tower: source,
            counts,
        } = command
            && *source == tower
        {
            for (available, selected) in stock.iter_mut().zip(counts) {
                *available = available.saturating_sub(*selected);
            }
        }
    }
    stock
}

fn context_panel(world: &mut World) {
    let selected = world.resource::<PlayerSelection>().entity;
    let mut recipe = world.resource::<PlayerSelection>().recipe;
    let owner = world
        .query_filtered::<&Allegiance, (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .next()
        .copied();
    let valid = selected.filter(|e| {
        world.get::<Health>(*e).is_some_and(|h| h.current > 0.0)
            && world.get::<Allegiance>(*e).copied() == owner
            && owner.is_some()
    });
    if selected != valid {
        world.resource_mut::<PlayerSelection>().entity = valid;
    }
    let mut element_rows = Vec::new();
    let mut anchor = Vec2::ZERO;
    let mut viewport = Vec2::new(320.0, 640.0);
    if let Some(entity) = valid {
        let position = world.get::<Position>(entity).unwrap().0;
        if let Some(window) = world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .iter(world)
            .next()
        {
            viewport = Vec2::new(window.width(), window.height());
        }
        if let Some((camera, transform)) = world
            .query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>()
            .iter(world)
            .next()
        {
            anchor = camera
                .world_to_viewport(transform, Vec3::new(position.x, 1.0, position.y))
                .unwrap_or(viewport * 0.5);
        }
        if world.get::<Tower>(entity).is_some() {
            let stock = world
                .get::<ElementInventory>(entity)
                .map_or([0; 5], |inventory| {
                    recipe_stock(inventory, entity, world.resource::<PlayerCommands>())
                });
            for (selected, available) in recipe.iter_mut().zip(stock) {
                *selected = (*selected).min(available);
            }
            world.resource_mut::<PlayerSelection>().recipe = recipe;
            let choices = Element::ALL
                .into_iter()
                .map(|element| {
                    let index = element as usize;
                    let mut choice = row(
                        format!("{}\n{}", element.name(), recipe[index]),
                        (recipe[index] < stock[index]).then_some(ContextAction::Recipe {
                            tower: entity,
                            element,
                        }),
                    );
                    choice.color = element.color();
                    (choice, recipe[index] > 0)
                })
                .collect();
            element_rows.push(choices);
        } else {
            let gear = world
                .get::<EquippedElements>(entity)
                .copied()
                .unwrap_or_default();
            let pending = world
                .query::<(&EquipmentOrder, &ElementInventory)>()
                .iter(world)
                .find(|(order, _)| order.minion == entity)
                .map(|(order, stock)| (*order, stock.counts()));
            let nearest = world.query_filtered::<(Entity, &Position, &Allegiance, &Health, &ElementInventory), With<Tower>>().iter(world)
                .filter(|(_, _, a, h, _)| Some(**a) == owner && h.current > 0.0)
                .min_by(|a, b| a.1.0.distance_squared(position).total_cmp(&b.1.0.distance_squared(position)))
                .map(|(e, _, _, _, stock)| (e, stock.counts()));
            for slot in 0..3 {
                let mut choices = Vec::new();
                for element in Element::ALL {
                    let equipped = gear.slots[slot] == Some(element);
                    let action = nearest.and_then(|(tower, stock)| {
                        let refunded = pending
                            .filter(|(order, _)| order.tower == tower)
                            .map_or(0, |(_, stock)| stock[element as usize]);
                        let command = if equipped {
                            PlayerCommand::ReturnEquipment {
                                minion: entity,
                                tower,
                                slot,
                            }
                        } else if stock[element as usize] > 0 || refunded > 0 {
                            PlayerCommand::Equip {
                                minion: entity,
                                tower,
                                slot,
                                element,
                            }
                        } else {
                            return None;
                        };
                        Some(ContextAction::Equipment {
                            command,
                            cancel: pending.is_some(),
                        })
                    });
                    let mut choice = row(element.name(), action);
                    choice.color = element.color();
                    choices.push((choice, equipped));
                }
                element_rows.push(choices);
            }
        }
    }
    let signature = format!(
        "{valid:?}:{recipe:?}:{}",
        element_rows
            .iter()
            .flatten()
            .map(|(r, selected)| format!("{}:{:?}:{selected}", r.label, r.action))
            .collect::<String>()
    );
    let width = 252.0_f32.min((viewport.x - 16.0).max(1.0));
    let height = 12.0 + element_rows.len() as f32 * 43.0 - 3.0;
    let left =
        (anchor.x + 20.0).clamp(8.0_f32.min(viewport.x), (viewport.x - width - 8.0).max(8.0));
    let top = (anchor.y - height * 0.5).clamp(
        8.0_f32.min(viewport.y),
        (viewport.y - height - 8.0).max(8.0),
    );
    if world.resource::<PanelCache>().0 == signature {
        for mut node in world
            .query_filtered::<&mut Node, With<ContextPanel>>()
            .iter_mut(world)
        {
            node.left = px(left);
            node.top = px(top);
            node.width = px(width);
            node.max_height = px(height);
        }
        return;
    }
    world.resource_mut::<PanelCache>().0 = signature;
    let old: Vec<_> = world
        .query_filtered::<Entity, With<ContextPanel>>()
        .iter(world)
        .collect();
    for entity in old {
        world.despawn(entity);
    }
    if valid.is_none() {
        return;
    }
    world
        .spawn((
            Name::new("Selection context panel"),
            ContextPanel,
            EntityKind::Ui,
            Interaction::None,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(width),
                max_height: px(height),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(6)),
                row_gap: px(3),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            GlobalZIndex(30),
            BackgroundColor(Color::srgba(0.035, 0.045, 0.065, 0.97)),
        ))
        .with_children(|panel| {
            for (slot, choices) in element_rows.into_iter().enumerate() {
                panel
                    .spawn((
                        EntityKind::Ui,
                        Name::new(format!("Element row {}", slot + 1)),
                        Node {
                            width: percent(100),
                            min_height: px(40),
                            flex_shrink: 0.0,
                            column_gap: px(3),
                            ..default()
                        },
                    ))
                    .with_children(|strip| {
                        for (choice, equipped) in choices {
                            let enabled = choice.action.is_some();
                            let mut button = strip.spawn((
                                EntityKind::Ui,
                                Name::new(format!("Element row {} {}", slot + 1, choice.label)),
                                Node {
                                    flex_basis: px(0),
                                    flex_grow: 1.0,
                                    min_width: px(0),
                                    min_height: px(40),
                                    border: UiRect::all(px(2)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BorderColor::all(if equipped { Color::WHITE } else { Color::NONE }),
                                BackgroundColor(if enabled {
                                    choice.color
                                } else {
                                    choice.color.with_alpha(0.22)
                                }),
                            ));
                            if let Some(action) = choice.action {
                                button.insert((Button, action));
                            }
                            button.with_children(|button| {
                                button.spawn((
                                    EntityKind::Ui,
                                    Name::new("Element choice label"),
                                    Text::new(choice.label),
                                    TextFont {
                                        font_size: FontSize::Px(11.0),
                                        ..default()
                                    },
                                    TextColor(if enabled {
                                        Color::WHITE
                                    } else {
                                        Color::srgb(0.55, 0.58, 0.62)
                                    }),
                                ));
                            });
                        }
                    });
            }
        });
}
