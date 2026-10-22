//! Friendly unit selection and contextual equipment/production controls.
use crate::{inspector::InspectorState, model::*};
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
struct PointerState {
    previous: Option<[(u64, Vec2); 2]>,
    suppress: bool,
    cooldown: f32,
}
#[derive(Component)]
struct ContextPanel;
#[derive(Component, Clone, Debug)]
enum ContextAction {
    Slot(usize),
    Recipe(usize, bool),
    Command(PlayerCommand),
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
    time: Res<Time>,
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
    pointer.cooldown = (pointer.cooldown - time.delta_secs()).max(0.0);
    if !active.is_empty()
        || touches.any_just_released()
        || touches.iter_just_canceled().next().is_some()
    {
        pointer.cooldown = 0.75;
    }
    if active.len() >= 2 {
        pointer.suppress = true;
    }
    if active.is_empty() {
        pointer.suppress = false;
    }
    if state.restart_requested
        || state.browser_hidden
        || state.winner.is_some()
        || inspector.is_some_and(|s| s.open)
        || interactions.iter().any(|(interaction, node, transform)| {
            if pointer.cooldown > 0.0 {
                let bounds = Rect::from_center_size(
                    transform.translation * node.inverse_scale_factor(),
                    node.size() * node.inverse_scale_factor(),
                );
                active.iter().any(|(_, p)| bounds.contains(*p))
            } else {
                *interaction != Interaction::None
            }
        })
    {
        pointer.previous = None;
        if !active.is_empty() {
            pointer.suppress = true;
        }
        return;
    }
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    if active.len() >= 2 {
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
    let position = touches
        .iter_just_pressed()
        .next()
        .map(|t| t.position())
        .or_else(|| {
            (pointer.cooldown <= 0.0 && mouse.just_pressed(MouseButton::Left))
                .then(|| windows.single().ok().and_then(Window::cursor_position))
                .flatten()
        });
    let Some(position) = position else {
        return;
    };
    let Some(owner) = player.iter().next() else {
        return;
    };
    selection.entity = entities
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
            .then_some((entity, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, _)| e);
}

fn context_actions(
    actions: Query<(&Interaction, &ContextAction), Changed<Interaction>>,
    mut selection: ResMut<PlayerSelection>,
    mut commands: ResMut<PlayerCommands>,
    state: Res<MatchState>,
) {
    if state.restart_requested || state.browser_hidden || state.winner.is_some() {
        return;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            ContextAction::Slot(slot) => selection.slot = *slot,
            ContextAction::Recipe(index, add) => {
                if *add {
                    selection.recipe[*index] += 1;
                } else {
                    selection.recipe[*index] = selection.recipe[*index].saturating_sub(1);
                }
            }
            ContextAction::Command(command) => commands.0.push(command.clone()),
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

fn context_panel(world: &mut World) {
    let selected = world.resource::<PlayerSelection>().entity;
    let slot = world.resource::<PlayerSelection>().slot.min(2);
    let recipe = world.resource::<PlayerSelection>().recipe;
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
    let mut rows = Vec::new();
    let mut heading = String::new();
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
            heading = "Tower stock / Create minion".into();
            let stock = world
                .get::<ElementInventory>(entity)
                .map_or([0; 5], ElementInventory::counts);
            let total: u32 = recipe.iter().sum();
            for element in Element::ALL {
                let i = element as usize;
                rows.push(row(
                    format!(
                        "{}: {} stock | {} selected",
                        element.name(),
                        stock[i],
                        recipe[i]
                    ),
                    None,
                ));
                rows.push(row(
                    format!("− {}", element.name()),
                    (recipe[i] > 0).then_some(ContextAction::Recipe(i, false)),
                ));
                rows.push(row(
                    format!("+ {}", element.name()),
                    (total < 5 && recipe[i] < stock[i]).then_some(ContextAction::Recipe(i, true)),
                ));
            }
            rows.push(row(
                format!("Create ({total}/5)"),
                (total == 5 && (0..5).all(|i| recipe[i] <= stock[i])).then_some(
                    ContextAction::Command(PlayerCommand::CreateMinion {
                        tower: entity,
                        counts: recipe,
                    }),
                ),
            ));
        } else {
            heading = "Minion equipment".into();
            let gear = world
                .get::<EquippedElements>(entity)
                .copied()
                .unwrap_or_default();
            let pending = world
                .query::<&EquipmentOrder>()
                .iter(world)
                .find(|o| o.minion == entity)
                .copied();
            rows.push(row(
                pending.map_or_else(
                    || "No pending tower order".into(),
                    |o| format!("Pending: {:?}", o.action),
                ),
                None,
            ));
            for i in 0..3 {
                rows.push(row(
                    format!(
                        "{} Slot {}: {}",
                        if slot == i { "›" } else { "" },
                        i + 1,
                        gear.slots[i].map_or("Empty", Element::name)
                    ),
                    Some(ContextAction::Slot(i)),
                ));
            }
            let nearest = world.query_filtered::<(Entity, &Position, &Allegiance, &Health, &ElementInventory), With<Tower>>().iter(world)
                .filter(|(_, _, a, h, _)| Some(**a) == owner && h.current > 0.0)
                .min_by(|a, b| a.1.0.distance_squared(position).total_cmp(&b.1.0.distance_squared(position)))
                .map(|(e, _, _, _, stock)| (e, stock.counts()));
            for element in Element::ALL {
                let available = nearest.map_or(0, |(_, stock)| stock[element as usize]);
                let mut choice = row(
                    format!("Equip {} ({available})", element.name()),
                    nearest
                        .filter(|_| available > 0 && pending.is_none())
                        .map(|(tower, _)| {
                            ContextAction::Command(PlayerCommand::Equip {
                                minion: entity,
                                tower,
                                slot,
                                element,
                            })
                        }),
                );
                choice.color = element.color();
                rows.push(choice);
            }
            rows.push(row(
                "Return selected equipment",
                nearest
                    .filter(|_| pending.is_none() && gear.slots[slot].is_some())
                    .map(|(tower, _)| {
                        ContextAction::Command(PlayerCommand::ReturnEquipment {
                            minion: entity,
                            tower,
                            slot,
                        })
                    }),
            ));
            let cargo = world
                .get::<ElementInventory>(entity)
                .is_some_and(|i| !i.is_empty());
            rows.push(row(
                "Deliver cargo",
                nearest
                    .filter(|_| pending.is_none() && cargo)
                    .map(|(tower, _)| {
                        ContextAction::Command(PlayerCommand::DeliverCargo {
                            minion: entity,
                            tower,
                        })
                    }),
            ));
            rows.push(row(
                "Cancel pending order",
                pending
                    .map(|_| ContextAction::Command(PlayerCommand::CancelOrder { minion: entity })),
            ));
        }
    }
    let signature = format!(
        "{valid:?}:{slot}:{recipe:?}:{heading}:{}",
        rows.iter()
            .map(|r| format!("{}:{:?}", r.label, r.action))
            .collect::<String>()
    );
    let width = 252.0_f32.min((viewport.x - 16.0).max(1.0));
    let row_height = ((viewport.y - 70.0) / rows.len().max(1) as f32 - 3.0).clamp(14.0, 26.0);
    let height = (rows.len() as f32 * (row_height + 3.0) + 42.0).min((viewport.y - 16.0).max(1.0));
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
            panel.spawn((
                EntityKind::Ui,
                Name::new("Selection heading"),
                Text::new(heading),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
            ));
            for row in rows {
                let enabled = row.action.is_some();
                let mut item = panel.spawn((
                    EntityKind::Ui,
                    Name::new("Selection action"),
                    Node {
                        min_height: px(row_height),
                        flex_shrink: 0.0,
                        padding: UiRect::axes(px(6), px(0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if enabled {
                        row.color
                    } else {
                        Color::srgb(0.075, 0.085, 0.10)
                    }),
                ));
                if let Some(action) = row.action {
                    item.insert((Button, action));
                }
                item.with_children(|button| {
                    button.spawn((
                        EntityKind::Ui,
                        Name::new("Selection action label"),
                        Text::new(row.label),
                        TextFont {
                            font_size: FontSize::Px((row_height - 3.0).clamp(10.0, 12.0)),
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
