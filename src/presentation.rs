//! Primitive-only presentation. Gameplay never depends on these mesh dimensions.
use crate::{influence::InfluenceField, inspector::InspectorState, model::*};
use bevy::{camera::ScalingMode, mesh::VertexAttributeValues, prelude::*};

pub struct PresentationPlugin;
impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, browser_visibility)
            .add_systems(
                Update,
                (
                    keyboard_controls,
                    button_controls,
                    attach_visuals,
                    sync_donut_visuals,
                    sync_visuals,
                    attach_resource_visuals,
                    sync_gathering_visuals,
                    sync_link_visuals,
                    ground_colors,
                    update_hud,
                    camera_controls,
                    browser_state,
                )
                    .chain(),
            );
    }
}
#[derive(Resource)]
struct VisualAssets {
    tower: Handle<Mesh>,
    minion: Handle<Mesh>,
    ground: Handle<Mesh>,
    deposit: Handle<Mesh>,
    trail: Handle<Mesh>,
    donut: Handle<Mesh>,
    link: Handle<Mesh>,
    raw: Handle<StandardMaterial>,
    trail_materials: Vec<Vec<Handle<StandardMaterial>>>,
    materials: Vec<Handle<StandardMaterial>>,
}
#[derive(Component)]
struct DonutVisual;
const TRAIL_BUCKETS: usize = 8;

#[derive(Component)]
struct SpectatorCamera;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct FactionText(usize);
#[derive(Component, Clone, Copy)]
enum Control {
    Pause,
    Speed(f32),
    Restart,
    Inspector,
}

fn setup(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tower = meshes.add(Cuboid::new(3.0, 7.0, 3.0));
    let minion = meshes.add(Capsule3d::new(0.42, 0.7));
    let ground = meshes.add(
        Plane3d::default()
            .mesh()
            .size(config.half_extent * 2.0, config.half_extent * 2.0)
            .subdivisions(96),
    );
    let palette = (0..5)
        .map(|i| {
            materials.add(StandardMaterial {
                base_color: faction_color(i),
                perceptual_roughness: 0.8,
                ..default()
            })
        })
        .collect();
    commands.spawn((
        Name::new("Battlefield"),
        EntityKind::Battlefield,
        Mesh3d(ground.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
    ));
    let trail_materials = (0..5)
        .map(|faction| {
            (0..TRAIL_BUCKETS)
                .map(|bucket| {
                    materials.add(StandardMaterial {
                        base_color: faction_color(faction)
                            .with_alpha((bucket + 1) as f32 / TRAIL_BUCKETS as f32 * 0.8),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    })
                })
                .collect()
        })
        .collect();
    commands.insert_resource(VisualAssets {
        tower,
        minion,
        ground,
        deposit: meshes.add(Torus::default()),
        trail: meshes.add(Cylinder::new(0.32, 0.08)),
        donut: meshes.add(Torus::default()),
        link: meshes.add(Cuboid::default()),
        raw: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 1.0, 0.8),
            unlit: true,
            ..default()
        }),
        trail_materials,
        materials: palette,
    });
    commands.spawn((
        Name::new("Sun"),
        EntityKind::Light,
        DirectionalLight {
            illuminance: 12000.0,
            ..default()
        },
        Transform::from_xyz(20.0, 50.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Spectator camera"),
        EntityKind::Camera,
        SpectatorCamera,
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::SomewhatBoringDisplayTransform,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 120.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(60.0, 90.0, 90.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands
        .spawn((
            Name::new("Match information"),
            EntityKind::Ui,
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(20),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(14)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.035, 0.055, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Game title"),
                EntityKind::Ui,
                Text::new("LORD OF TOWERS"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
            ));
            parent.spawn((
                Name::new("Match status"),
                EntityKind::Ui,
                StatusText,
                Text::new("Starting simulation..."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
            ));
            for (i, name) in FACTION_NAMES.iter().enumerate() {
                parent.spawn((
                    Name::new(format!("{name} statistics")),
                    EntityKind::Ui,
                    FactionText(i),
                    Text::new(*name),
                    TextColor(faction_color(i)),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                ));
            }
            parent.spawn((
                Name::new("Camera instructions"),
                EntityKind::Ui,
                Text::new(
                    "Pan: WASD / arrows   Zoom: + / -\nSpace: pause   1/2/3: speed   R: restart",
                ),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.7, 0.8)),
            ));
            parent.spawn((
                Name::new("Build identifier"),
                EntityKind::Ui,
                Text::new(format!(
                    "Build {}",
                    option_env!("BUILD_COMMIT")
                        .unwrap_or("development")
                        .chars()
                        .take(7)
                        .collect::<String>()
                )),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
            ));
        });
    commands
        .spawn((
            Name::new("Spectator controls"),
            EntityKind::Ui,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(20),
                left: px(20),
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8),
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (label, control) in [
                ("Pause / Play", Control::Pause),
                ("1x", Control::Speed(1.0)),
                ("2x", Control::Speed(2.0)),
                ("4x", Control::Speed(4.0)),
                ("Restart", Control::Restart),
                ("Entities", Control::Inspector),
            ] {
                parent
                    .spawn((
                        Name::new(label),
                        EntityKind::Ui,
                        Button,
                        control,
                        Node {
                            padding: UiRect::axes(px(14), px(12)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.13, 0.17, 0.23)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Name::new(format!("{label} label")),
                            EntityKind::Ui,
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                        ));
                    });
            }
        });
}
fn attach_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    entities: Query<
        (Entity, &Allegiance, Option<&Tower>),
        (Or<(With<Tower>, With<Minion>)>, Without<Mesh3d>),
    >,
) {
    for (entity, allegiance, tower) in &entities {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                Mesh3d(if tower.is_some() {
                    assets.tower.clone()
                } else {
                    assets.minion.clone()
                }),
                MeshMaterial3d(assets.materials[faction.index].clone()),
                Transform::default(),
            ));
        }
    }
}
// Eight pieces per tier keep each unit count visible without scaling by inventory.
fn donut_transform(index: usize, tower: bool) -> Transform {
    let angle = (index % 8) as f32 * std::f32::consts::TAU / 8.0;
    let radius = if tower { 1.3 } else { 0.65 };
    let base = if tower { 3.72 } else { 1.0 };
    Transform::from_xyz(
        angle.cos() * radius,
        base + (index / 8) as f32 * 0.24,
        angle.sin() * radius,
    )
    .with_scale(Vec3::splat(0.18))
}

// Exclusive reconciliation makes additions/removals visible in this same frame.
fn sync_donut_visuals(world: &mut World) {
    let assets = world.resource::<VisualAssets>();
    let mesh = assets.donut.clone();
    let material = assets.raw.clone();
    let inventories: Vec<_> = world
        .query_filtered::<(Entity, &RawElement, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>()
        .iter(world)
        .map(|(entity, raw, tower)| (entity, raw.0 as usize, tower.is_some()))
        .collect();
    for (parent, count, tower) in inventories {
        let mut children: Vec<Entity> = world
            .get::<Children>(parent)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.get::<DonutVisual>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default();
        let retained = count.min(children.len());
        for child in children.drain(retained..) {
            world.despawn(child);
        }
        for (index, child) in children.iter().enumerate() {
            world
                .entity_mut(*child)
                .insert(donut_transform(index, tower));
        }
        for index in children.len()..count {
            world.spawn((
                Name::new(format!("Crown donut {}", index + 1)),
                EntityKind::DonutVisual,
                DonutVisual,
                ChildOf(parent),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                donut_transform(index, tower),
            ));
        }
    }
}
fn sync_visuals(
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    mut entities: Query<
        (
            &Position,
            &Allegiance,
            Option<&Tower>,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
) {
    for (position, allegiance, tower, mut transform, mut material) in &mut entities {
        transform.translation = Vec3::new(
            position.0.x,
            if tower.is_some() { 3.5 } else { 0.77 },
            position.0.y,
        );
        if let Ok(faction) = factions.get(allegiance.0) {
            let handle = &assets.materials[faction.index];
            if material.0 != *handle {
                material.0 = handle.clone();
            }
        }
    }
}
fn attach_resource_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    deposits: Query<Entity, (With<RawElementDeposit>, Without<Mesh3d>)>,
    trails: Query<(Entity, &Allegiance), (With<Trail>, Without<Mesh3d>)>,
    links: Query<(Entity, &Allegiance), (With<Link>, Without<Mesh3d>)>,
    factions: Query<&Faction>,
) {
    for entity in &deposits {
        commands.entity(entity).insert((
            Mesh3d(assets.deposit.clone()),
            MeshMaterial3d(assets.raw.clone()),
            Transform::default(),
        ));
    }
    for (entity, allegiance) in &trails {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                Mesh3d(assets.trail.clone()),
                MeshMaterial3d(assets.trail_materials[faction.index][TRAIL_BUCKETS - 1].clone()),
                Transform::default(),
            ));
        }
    }
    for (entity, allegiance) in &links {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                Mesh3d(assets.link.clone()),
                MeshMaterial3d(assets.materials[faction.index].clone()),
                Transform::default(),
                Visibility::Inherited,
            ));
        }
    }
}
fn trail_bucket(remaining: f32, lifetime: f32) -> usize {
    ((remaining / lifetime.max(f32::EPSILON)).clamp(0.0, 1.0) * TRAIL_BUCKETS as f32)
        .ceil()
        .max(1.0) as usize
        - 1
}
fn sync_gathering_visuals(
    assets: Res<VisualAssets>,
    config: Res<SimulationConfig>,
    factions: Query<&Faction>,
    mut deposits: Query<(&Position, &mut Transform), (With<RawElementDeposit>, Without<Trail>)>,
    mut trails: Query<
        (
            &Position,
            &Trail,
            &Allegiance,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<RawElementDeposit>, Without<DonutVisual>),
    >,
) {
    for (position, mut transform) in &mut deposits {
        transform.translation = Vec3::new(position.0.x, 0.35, position.0.y);
        transform.scale = Vec3::splat(0.55);
    }
    for (position, trail, allegiance, mut transform, mut material) in &mut trails {
        transform.translation = Vec3::new(position.0.x, 0.1, position.0.y);
        if let Ok(faction) = factions.get(allegiance.0) {
            material.0 = assets.trail_materials[faction.index]
                [trail_bucket(trail.remaining, config.trail_lifetime)]
            .clone();
        }
    }
}
// Stretch a unit cuboid along local X. Gameplay endpoints remain independent of mesh size.
fn link_transform(source: Vec2, target: Vec2) -> Option<Transform> {
    let delta = Vec3::new(target.x - source.x, 0.0, target.y - source.y);
    let length = delta.length();
    if !length.is_finite() || length < 0.001 {
        return None;
    }
    Some(Transform {
        translation: Vec3::new(
            (source.x + target.x) * 0.5,
            0.12,
            (source.y + target.y) * 0.5,
        ),
        rotation: Quat::from_rotation_arc(Vec3::X, delta / length),
        scale: Vec3::new(length, 0.055, 0.055),
    })
}
fn sync_link_visuals(
    assets: Res<VisualAssets>,
    positions: Query<&Position>,
    factions: Query<&Faction>,
    mut links: Query<(
        &Link,
        &Allegiance,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (link, allegiance, mut transform, mut visibility, mut material) in &mut links {
        let endpoints = positions
            .get(link.source)
            .ok()
            .zip(positions.get(link.target).ok());
        let pose = endpoints.and_then(|(source, target)| link_transform(source.0, target.0));
        let faction = factions.get(allegiance.0).ok();
        if let (Some(pose), Some(faction)) = (pose, faction) {
            *transform = pose;
            *visibility = Visibility::Inherited;
            let handle = &assets.materials[faction.index];
            if material.0 != *handle {
                material.0 = handle.clone();
            }
        } else {
            // Endpoints can disappear during combat/restart before graph reconciliation.
            *visibility = Visibility::Hidden;
        }
    }
}
fn ground_colors(
    field: Res<InfluenceField>,
    assets: Res<VisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut revision: Local<Option<u64>>,
) {
    if *revision == Some(field.revision) {
        return;
    }
    let Some(mut mesh) = meshes.get_mut(&assets.ground) else {
        return;
    };
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let colors: Vec<[f32; 4]> = positions
        .iter()
        .map(|p| {
            let influence = field.sample(Vec2::new(p[0], p[2]));
            let mut ordered: Vec<(usize, f32)> = influence.iter().copied().enumerate().collect();
            ordered.sort_by(|a, b| b.1.total_cmp(&a.1));
            let (winner, strength) = ordered[0];
            let blend = ((strength - ordered[1].1) / strength.max(0.01)).clamp(0.0, 1.0)
                * (strength * 0.5).clamp(0.0, 0.65);
            let c = faction_color(winner).to_linear();
            [
                0.07 + (c.red - 0.07) * blend,
                0.09 + (c.green - 0.09) * blend,
                0.11 + (c.blue - 0.11) * blend,
                1.0,
            ]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    *revision = Some(field.revision);
}
fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<MatchState>) {
    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }
    for (key, speed) in [
        (KeyCode::Digit1, 1.0),
        (KeyCode::Digit2, 2.0),
        (KeyCode::Digit3, 4.0),
    ] {
        if keys.just_pressed(key) {
            state.speed = speed;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.restart_requested = true;
    }
}
fn button_controls(
    mut buttons: Query<(&Interaction, &Control, &mut BackgroundColor), Changed<Interaction>>,
    mut state: ResMut<MatchState>,
    mut inspector: ResMut<InspectorState>,
) {
    for (interaction, control, mut color) in &mut buttons {
        color.0 = if *interaction == Interaction::None {
            Color::srgb(0.13, 0.17, 0.23)
        } else {
            Color::srgb(0.24, 0.32, 0.43)
        };
        if *interaction == Interaction::Pressed {
            match control {
                Control::Pause => state.paused = !state.paused,
                Control::Speed(speed) => state.speed = *speed,
                Control::Restart => state.restart_requested = true,
                Control::Inspector => inspector.open = !inspector.open,
            }
        }
    }
}
fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<SpectatorCamera>>,
) {
    for (mut transform, mut projection) in &mut cameras {
        let x = (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) as i32
            - (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) as i32;
        let z = (keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)) as i32
            - (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)) as i32;
        transform.translation +=
            Vec3::new(x as f32, 0.0, z as f32) * time.delta_secs().min(0.1) * 40.0;
        if let Projection::Orthographic(ref mut orthographic) = *projection {
            let zoom = (keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract))
                as i32
                - (keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd)) as i32;
            orthographic.scale =
                (orthographic.scale + zoom as f32 * time.delta_secs().min(0.1)).clamp(0.3, 2.0);
        }
    }
}
fn update_hud(
    state: Res<MatchState>,
    field: Res<InfluenceField>,
    factions: Query<&Faction>,
    units: Query<
        (&Allegiance, Option<&Tower>, Option<&RawElement>),
        Or<(With<Tower>, With<Minion>)>,
    >,
    mut texts: Query<(&mut Text, Option<&FactionText>), Or<(With<StatusText>, With<FactionText>)>>,
) {
    let mut towers = [0; 5];
    let mut minions = [0; 5];
    let mut stock = [0; 5];
    for (allegiance, tower, raw) in &units {
        if let Ok(faction) = factions.get(allegiance.0) {
            if tower.is_some() {
                towers[faction.index] += 1;
                stock[faction.index] += raw.map_or(0, |raw| raw.0);
            } else {
                minions[faction.index] += 1;
            }
        }
    }
    let territory = field.territory_percentages();
    for (mut text, faction) in &mut texts {
        text.0 = if let Some(FactionText(i)) = faction {
            format!(
                "{}  |  {} towers | {} minions | rawElement {} | {:.0}%",
                FACTION_NAMES[*i], towers[*i], minions[*i], stock[*i], territory[*i]
            )
        } else if let Some(winner) = state.winner {
            format!("{} wins! Restart to play again.", FACTION_NAMES[winner])
        } else {
            format!(
                "{} | {:.0}x speed | {:.0}s",
                if state.paused { "Paused" } else { "Expanding" },
                state.speed,
                state.elapsed
            )
        };
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn browser_state() {}
#[cfg(target_arch = "wasm32")]
fn browser_state(
    state: Res<MatchState>,
    inspector: Res<InspectorState>,
    minions: Query<(), With<Minion>>,
    towers: Query<(), With<Tower>>,
    deposits: Query<&RawElement, With<RawElementDeposit>>,
    crown: Query<(), With<DonutVisual>>,
    inventories: Query<(&RawElement, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>,
    trails: Query<(), With<Trail>>,
    economy: Res<RawElementEconomy>,
    links: Query<(), With<Link>>,
    stores: Query<&EnergyStore, With<Minion>>,
    clock: Res<EnergyClock>,
) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(loading) = document.get_element_by_id("loading") {
        let _ = loading.set_attribute("hidden", "");
    }
    if let Some(status) = document.get_element_by_id("game-status") {
        for (key, value) in [
            ("data-ready", "true".to_string()),
            ("data-minions", minions.iter().count().to_string()),
            ("data-towers", towers.iter().count().to_string()),
            ("data-paused", state.paused.to_string()),
            ("data-speed", state.speed.to_string()),
            ("data-elapsed", state.elapsed.to_string()),
            ("data-inspector", inspector.open.to_string()),
            ("data-raw-deposits", deposits.iter().count().to_string()),
            ("data-donut-visuals", crown.iter().count().to_string()),
            (
                "data-raw-remaining",
                deposits
                    .iter()
                    .map(|raw| u64::from(raw.0))
                    .sum::<u64>()
                    .to_string(),
            ),
            (
                "data-raw-carried",
                inventories
                    .iter()
                    .filter(|(_, tower)| tower.is_none())
                    .map(|(raw, _)| u64::from(raw.0))
                    .sum::<u64>()
                    .to_string(),
            ),
            (
                "data-raw-stock",
                inventories
                    .iter()
                    .filter(|(_, tower)| tower.is_some())
                    .map(|(raw, _)| u64::from(raw.0))
                    .sum::<u64>()
                    .to_string(),
            ),
            ("data-raw-spent", economy.spent.to_string()),
            ("data-raw-initial", economy.initial.to_string()),
            ("data-trails", trails.iter().count().to_string()),
            ("data-links", links.iter().count().to_string()),
            (
                "data-total-minion-energy",
                stores
                    .iter()
                    .map(|store| u64::from(store.current))
                    .sum::<u64>()
                    .to_string(),
            ),
            ("data-energy-pulses", clock.pulses.to_string()),
        ] {
            let _ = status.set_attribute(key, &value);
        }
    }
}

// The page records visibility transitions even while requestAnimationFrame is suspended.
// Consume the resume latch before FixedMain so hidden wall time never advances the match.
#[cfg(not(target_arch = "wasm32"))]
fn browser_visibility() {}
#[cfg(target_arch = "wasm32")]
fn browser_visibility(mut state: ResMut<MatchState>) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let resumed = document
        .get_element_by_id("game-status")
        .is_some_and(|status| {
            let resumed = status.get_attribute("data-resumed").as_deref() == Some("true");
            if resumed {
                let _ = status.remove_attribute("data-resumed");
            }
            resumed
        });
    state.browser_hidden = document.hidden() || resumed;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn donuts(world: &mut World, parent: Entity) -> Vec<Entity> {
        world
            .query_filtered::<(Entity, &ChildOf), With<DonutVisual>>()
            .iter(world)
            .filter_map(|(entity, child_of)| (child_of.parent() == parent).then_some(entity))
            .collect()
    }

    #[test]
    fn crown_tiers_are_above_unit_tops_and_repeat_every_eight() {
        for tower in [false, true] {
            let first = donut_transform(0, tower);
            let ninth = donut_transform(8, tower);
            assert!((ninth.translation.y - first.translation.y - 0.24).abs() < 0.0001);
            assert_eq!(first.translation.x, ninth.translation.x);
            assert_eq!(first.translation.z, ninth.translation.z);
            assert!(first.translation.y > if tower { 3.5 } else { 0.77 });
            assert_ne!(first.translation, donut_transform(1, tower).translation);
        }
    }

    #[test]
    fn link_mesh_spans_endpoints_and_rejects_degenerate_segments() {
        let source = Vec2::new(-3.0, 7.0);
        let target = Vec2::new(4.0, -2.0);
        let transform = link_transform(source, target).unwrap();
        let start = transform.transform_point(Vec3::NEG_X * 0.5);
        let end = transform.transform_point(Vec3::X * 0.5);
        assert!(start.distance(Vec3::new(source.x, 0.12, source.y)) < 0.0001);
        assert!(end.distance(Vec3::new(target.x, 0.12, target.y)) < 0.0001);
        assert!(link_transform(source, source).is_none());
        assert!(link_transform(Vec2::splat(f32::NAN), target).is_none());
    }

    #[test]
    fn trail_fade_uses_bounded_buckets() {
        assert_eq!(trail_bucket(30.0, 30.0), TRAIL_BUCKETS - 1);
        assert_eq!(trail_bucket(15.0, 30.0), 3);
        assert_eq!(trail_bucket(0.0, 30.0), 0);
        assert_eq!(trail_bucket(100.0, 30.0), TRAIL_BUCKETS - 1);
    }
    #[test]
    fn gathering_visuals_share_assets_and_donuts_follow_inventory_and_parent() {
        let mut app = App::new();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let raw = materials.add(StandardMaterial::from(Color::WHITE));
        let palette: Vec<_> = (0..5)
            .map(|i| materials.add(StandardMaterial::from(faction_color(i))))
            .collect();
        let fades: Vec<Vec<_>> = (0..5)
            .map(|_| {
                (0..TRAIL_BUCKETS)
                    .map(|_| materials.add(StandardMaterial::default()))
                    .collect()
            })
            .collect();
        app.insert_resource(VisualAssets {
            tower: meshes.add(Cuboid::default()),
            minion: meshes.add(Capsule3d::default()),
            ground: meshes.add(Plane3d::default()),
            deposit: meshes.add(Torus::default()),
            trail: meshes.add(Cylinder::default()),
            donut: meshes.add(Torus::default()),
            link: meshes.add(Cuboid::default()),
            raw: raw.clone(),
            trail_materials: fades.clone(),
            materials: palette.clone(),
        });
        app.init_resource::<SimulationConfig>();
        app.add_systems(
            Update,
            (
                attach_visuals,
                sync_donut_visuals,
                sync_visuals,
                attach_resource_visuals,
                sync_gathering_visuals,
                sync_link_visuals,
            )
                .chain(),
        );
        let own = app.world_mut().spawn(Faction { index: 0 }).id();
        let enemy = app.world_mut().spawn(Faction { index: 1 }).id();
        let minion = app
            .world_mut()
            .spawn((Minion, Position(Vec2::ZERO), Allegiance(own), RawElement(0)))
            .id();
        let tower = app
            .world_mut()
            .spawn((
                Tower { slot: 0 },
                Position(Vec2::X * 2.0),
                Allegiance(own),
                RawElement(9),
                MatchEntity,
            ))
            .id();
        let deposit = app
            .world_mut()
            .spawn((RawElementDeposit, Position(Vec2::X * 3.0), RawElement(5)))
            .id();
        let trail = app
            .world_mut()
            .spawn((
                Trail {
                    trip: 0,
                    sequence: 0,
                    remaining: 15.0,
                },
                Position(Vec2::X),
                Allegiance(own),
            ))
            .id();
        let link = app
            .world_mut()
            .spawn((
                Link {
                    source: tower,
                    target: minion,
                },
                Allegiance(own),
            ))
            .id();
        app.world_mut().run_schedule(Update);
        assert_eq!(donuts(app.world_mut(), minion).len(), 0);
        let crown = donuts(app.world_mut(), tower);
        assert_eq!(crown.len(), 9);
        let mesh = app.world().resource::<VisualAssets>().donut.clone();
        for child in &crown {
            assert_eq!(app.world().get::<Mesh3d>(*child).unwrap().0, mesh);
            assert_eq!(
                app.world()
                    .get::<MeshMaterial3d<StandardMaterial>>(*child)
                    .unwrap()
                    .0,
                raw
            );
            assert_eq!(
                *app.world().get::<EntityKind>(*child).unwrap(),
                EntityKind::DonutVisual
            );
            assert!(app.world().get::<RawElement>(*child).is_none());
            assert!(app.world().get::<MatchEntity>(*child).is_none());
        }
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(deposit)
                .unwrap()
                .0,
            raw
        );
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(trail)
                .unwrap()
                .0,
            fades[0][3]
        );
        app.world_mut().get_mut::<RawElement>(minion).unwrap().0 = 1;
        app.world_mut().entity_mut(tower).insert(Allegiance(enemy));
        app.world_mut().run_schedule(Update);
        let carried = donuts(app.world_mut(), minion);
        assert_eq!(carried.len(), 1);
        assert_eq!(donuts(app.world_mut(), tower), crown);
        app.world_mut().get_mut::<RawElement>(tower).unwrap().0 = 1;
        app.world_mut().run_schedule(Update);
        assert_eq!(donuts(app.world_mut(), tower).len(), 1);
        assert!(
            crown
                .iter()
                .skip(1)
                .all(|child| app.world().get_entity(*child).is_err())
        );
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(tower)
                .unwrap()
                .0,
            palette[1]
        );
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(minion)
                .unwrap()
                .0,
            palette[0]
        );
        app.world_mut().get_mut::<RawElement>(minion).unwrap().0 = 0;
        app.world_mut().run_schedule(Update);
        assert!(donuts(app.world_mut(), minion).is_empty());
        assert!(app.world().get_entity(carried[0]).is_err());
        app.world_mut().get_mut::<RawElement>(minion).unwrap().0 = 17;
        app.world_mut().run_schedule(Update);
        let carried = donuts(app.world_mut(), minion);
        assert_eq!(carried.len(), 17);
        app.world_mut().despawn(minion);
        app.world_mut().run_schedule(Update);
        assert!(
            carried
                .iter()
                .all(|child| app.world().get_entity(*child).is_err())
        );
        let remaining = donuts(app.world_mut(), tower);
        app.world_mut().despawn(tower);
        assert!(
            remaining
                .iter()
                .all(|child| app.world().get_entity(*child).is_err())
        );
        assert_eq!(
            *app.world().get::<Visibility>(link).unwrap(),
            Visibility::Hidden
        );
    }
}
