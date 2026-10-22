//! Primitive-only presentation. Gameplay never depends on these mesh dimensions.
use crate::{influence::InfluenceField, inspector::InspectorState, model::*};
use bevy::{camera::ScalingMode, mesh::VertexAttributeValues, prelude::*};

pub struct PresentationPlugin;
impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
            .init_resource::<TrailTool>()
            .init_resource::<HudVisibility>()
            .init_resource::<CameraGesture>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, browser_visibility)
            .add_systems(
                PostUpdate,
                browser_control_positions.after(bevy::ui::UiSystems::Layout),
            )
            .add_systems(
                Update,
                (
                    responsive_layout,
                    browser_mobile_state.after(camera_controls),
                ),
            )
            .add_systems(
                Update,
                (
                    keyboard_controls,
                    button_controls,
                    attach_visuals,
                    sync_stacks,
                    sync_donut_visuals,
                    sync_energy_orbs,
                    attach_projectile_visuals,
                    sync_projectile_visuals,
                    sync_visuals,
                    attach_resource_visuals,
                    sync_gathering_visuals,
                    sync_tower_influence_visuals,
                    sync_control_state,
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
    influence_ring: Handle<Mesh>,
    influence_materials: Vec<Handle<StandardMaterial>>,
    energy: Handle<Mesh>,
    raw: Handle<StandardMaterial>,
    trail_materials: Vec<Vec<Handle<StandardMaterial>>>,
    materials: Vec<Handle<StandardMaterial>>,
}
#[derive(Component)]
struct DonutVisual;
#[derive(Component)]
struct EnergyOrbVisual;
const TRAIL_BUCKETS: usize = 8;

#[derive(Component)]
struct SpectatorCamera;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct HudRoot;
#[derive(Component)]
struct HudTitle;
#[derive(Component)]
struct HudToggleText;
#[derive(Component)]
struct DesktopDetail;
#[derive(Component)]
struct ControlBar;
fn mobile_layout(width: f32, height: f32) -> bool {
    width < 760.0 || height < 540.0
}
#[derive(Component)]
struct FactionText(usize);
#[derive(Component, Clone, Copy)]
enum Control {
    Pause,
    Speed(f32),
    Restart,
    Hud,
    Tool(TrailTool),
}

fn setup(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tower = meshes.add(Cuboid::new(0.62, 0.62, 0.62));
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
        influence_ring: meshes.add(Torus::new(0.99, 1.0)),
        influence_materials: (0..5)
            .map(|i| {
                materials.add(StandardMaterial {
                    base_color: faction_color(i).with_alpha(0.3),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })
            })
            .collect(),
        energy: meshes.add(Sphere::new(0.2)),
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
            scaling_mode: ScalingMode::AutoMin {
                min_width: 120.0,
                min_height: 120.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(60.0, 90.0, 90.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands
        .spawn((
            Name::new("Match information"),
            HudRoot,
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
                HudTitle,
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
                    DesktopDetail,
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
                DesktopDetail,
                EntityKind::Ui,
                Text::new(
                    "Pan: WASD / arrows   Zoom: + / -\nSpace: pause   1/2/3: speed   R: restart\nDrag/touch: draw trail   Right click near trail: erase",
                ),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.7, 0.8)),
            ));
            parent.spawn((
                Name::new("Build identifier"),
                DesktopDetail,
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
            ControlBar,
            EntityKind::Ui,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(20),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8),
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (label, control) in [
                ("Pause", Control::Pause),
                ("Draw", Control::Tool(TrailTool::Draw)),
                ("Erase", Control::Tool(TrailTool::Erase)),
                ("1x", Control::Speed(1.0)),
                ("2x", Control::Speed(2.0)),
                ("4x", Control::Speed(4.0)),
                ("Restart", Control::Restart),
            ] {
                parent
                    .spawn((
                        Name::new(label),
                        EntityKind::Ui,
                        Button,
                        control,
                        Node {
                            min_width: px(44),
                            height: px(44),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::axes(px(14), px(0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.13, 0.17, 0.23)),
                    ))
                    .with_children(|button| {
                        if matches!(control, Control::Pause) {
                            spawn_pause_icon(button);
                            return;
                        }
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
    commands
        .spawn((
            Name::new("Hide HUD"),
            EntityKind::Ui,
            Button,
            Control::Hud,
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                right: px(8),
                width: px(96),
                height: px(44),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(10),
            BackgroundColor(Color::srgb(0.13, 0.17, 0.23)),
        ))
        .with_children(|button| {
            button.spawn((
                Name::new("HUD visibility label"),
                EntityKind::Ui,
                HudToggleText,
                Text::new("Hide HUD"),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
            ));
        });
}
fn attach_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    entities: Query<
        (Entity, &Allegiance, Option<&Tower>),
        (Or<(With<Tower>, With<Minion>)>, Without<Transform>),
    >,
) {
    for (entity, allegiance, tower) in &entities {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                MeshMaterial3d(assets.materials[faction.index].clone()),
                Transform::default(),
                Visibility::Inherited,
            ));
            if tower.is_none() {
                commands
                    .entity(entity)
                    .insert(Mesh3d(assets.minion.clone()));
            }
        }
    }
}
/// Reusable primitive stack: count, grid layout, and spacing are presentation only.
#[derive(Component, Clone, Copy, Debug)]
struct Stack {
    count: usize,
    columns: usize,
    spacing: Vec3,
}
#[derive(Component)]
struct StackPiece;
#[derive(Component)]
struct TowerBeacon;

impl Stack {
    fn transform(self, index: usize) -> Transform {
        let width = self.columns;
        let layer = width * width;
        Transform::from_xyz(
            (index % width) as f32 * self.spacing.x - (width - 1) as f32 * self.spacing.x / 2.0,
            (index / layer) as f32 * self.spacing.y + self.spacing.y / 2.0,
            ((index / width) % width) as f32 * self.spacing.z
                - (width - 1) as f32 * self.spacing.z / 2.0,
        )
    }
}
fn health_count(health: Option<&Health>) -> usize {
    health.map_or(0, |health| health.current.max(0.0).floor() as usize)
}
fn tower_top(health: Option<&Health>) -> f32 {
    health_count(health).div_ceil(16) as f32 * 0.68
}
fn sync_stacks(world: &mut World) {
    let parents: Vec<_> = world
        .query_filtered::<(
            Entity,
            Option<&Tower>,
            Option<&Health>,
            Option<&RawElement>,
            Option<&Allegiance>,
        ), Or<(With<Tower>, With<RawElementDeposit>)>>()
        .iter(world)
        .map(|(entity, tower, health, raw, allegiance)| {
            (
                entity,
                tower.is_some(),
                health_count(health),
                raw.map_or(0, |raw| raw.0 as usize),
                allegiance.copied(),
            )
        })
        .collect();
    for (parent, tower, health, raw, allegiance) in parents {
        let stack = Stack {
            count: if tower { health } else { raw },
            columns: if tower { 4 } else { 1 },
            spacing: if tower {
                Vec3::splat(0.68)
            } else {
                Vec3::new(0.0, 0.24, 0.0)
            },
        };
        world.entity_mut(parent).insert(stack);
        let assets = world.resource::<VisualAssets>();
        let mesh = if tower {
            assets.tower.clone()
        } else {
            assets.deposit.clone()
        };
        let material = allegiance
            .and_then(|a| world.get::<Faction>(a.0))
            .map_or_else(|| assets.raw.clone(), |f| assets.materials[f.index].clone());
        let beacon_mesh = assets.energy.clone();
        let children: Vec<_> = world
            .get::<Children>(parent)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.get::<StackPiece>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default();
        for child in children.iter().skip(stack.count) {
            world.despawn(*child);
        }
        for index in 0..stack.count {
            let transform = stack.transform(index);
            if let Some(child) = children.get(index) {
                world
                    .entity_mut(*child)
                    .insert((transform, MeshMaterial3d(material.clone())));
            } else {
                world.spawn((
                    Name::new("Stack primitive"),
                    EntityKind::StackVisual,
                    StackPiece,
                    ChildOf(parent),
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    transform,
                ));
            }
        }
        if tower {
            let beacon = world.get::<Children>(parent).and_then(|children| {
                children
                    .iter()
                    .find(|child| world.get::<TowerBeacon>(*child).is_some())
            });
            let transform = Transform::from_xyz(0.0, health.div_ceil(16) as f32 * 0.68 + 0.45, 0.0)
                .with_scale(Vec3::splat(2.0));
            if let Some(beacon) = beacon {
                world
                    .entity_mut(beacon)
                    .insert((transform, MeshMaterial3d(material)));
            } else {
                world.spawn((
                    Name::new("Tower beacon"),
                    EntityKind::TowerBeacon,
                    TowerBeacon,
                    ChildOf(parent),
                    Mesh3d(beacon_mesh),
                    MeshMaterial3d(material),
                    transform,
                ));
            }
        }
    }
}
// Eight pieces per tier keep each unit count visible without scaling by inventory.
fn donut_transform(index: usize, tower: bool) -> Transform {
    let angle = (index % 8) as f32 * std::f32::consts::TAU / 8.0;
    let radius = if tower { 1.3 } else { 0.65 };
    let base = if tower { 1.0 } else { 1.15 };
    Transform::from_xyz(
        angle.cos() * radius,
        base + (index / 8) as f32 * 0.24,
        angle.sin() * radius,
    )
    .with_scale(Vec3::splat(0.36))
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
        let offset = if tower {
            tower_top(world.get::<Health>(parent)) + 0.6
        } else {
            0.0
        };
        for (index, child) in children.iter().enumerate() {
            let mut transform = donut_transform(index, tower);
            transform.translation.y += offset;
            world.entity_mut(*child).insert(transform);
        }
        for index in children.len()..count {
            world.spawn((
                Name::new(format!("Crown donut {}", index + 1)),
                EntityKind::DonutVisual,
                DonutVisual,
                ChildOf(parent),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                {
                    let mut transform = donut_transform(index, tower);
                    transform.translation.y += offset;
                    transform
                },
            ));
        }
    }
}
// Keep energy separate from the raw-element crown, in rings around the body.
fn energy_orb_transform(index: usize) -> Transform {
    let angle = (index % 8) as f32 * std::f32::consts::TAU / 8.0;
    Transform::from_xyz(
        angle.cos() * 0.95,
        0.15 + (index / 8) as f32 * 0.45,
        angle.sin() * 0.95,
    )
}

fn sync_energy_orbs(world: &mut World) {
    let mesh = world.resource::<VisualAssets>().energy.clone();
    let inventories: Vec<_> = world
        .query_filtered::<(Entity, &EnergyStore, &Allegiance), With<Minion>>()
        .iter(world)
        .map(|(entity, store, allegiance)| (entity, usize::from(store.current), allegiance.0))
        .collect();
    for (parent, count, faction_entity) in inventories {
        let Some(faction) = world.get::<Faction>(faction_entity) else {
            continue;
        };
        let material = world.resource::<VisualAssets>().materials[faction.index].clone();
        let mut children: Vec<_> = world
            .get::<Children>(parent)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| world.get::<EnergyOrbVisual>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default();
        let retained = count.min(children.len());
        for child in children.drain(retained..) {
            world.despawn(child);
        }
        for (index, child) in children.iter().enumerate() {
            world.entity_mut(*child).insert((
                energy_orb_transform(index),
                MeshMaterial3d(material.clone()),
            ));
        }
        for index in children.len()..count {
            world.spawn((
                Name::new(format!("Stored energy sphere {}", index + 1)),
                EntityKind::EnergyOrbVisual,
                EnergyOrbVisual,
                ChildOf(parent),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                energy_orb_transform(index),
            ));
        }
    }
}

fn attach_projectile_visuals(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    projectiles: Query<(Entity, &Allegiance), (With<EnergyProjectile>, Without<Mesh3d>)>,
) {
    for (entity, allegiance) in &projectiles {
        if let Ok(faction) = factions.get(allegiance.0) {
            commands.entity(entity).insert((
                Mesh3d(assets.energy.clone()),
                MeshMaterial3d(assets.materials[faction.index].clone()),
                Transform::default(),
            ));
        }
    }
}

fn sync_projectile_visuals(
    mut projectiles: Query<(&Position, &mut Transform), With<EnergyProjectile>>,
) {
    for (position, mut transform) in &mut projectiles {
        transform.translation = Vec3::new(position.0.x, 1.0, position.0.y);
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
            if tower.is_some() { 0.0 } else { 0.77 },
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
    deposits: Query<Entity, (With<RawElementDeposit>, Without<Transform>)>,
    trails: Query<(Entity, &Allegiance), (With<Trail>, Without<Mesh3d>)>,
    factions: Query<&Faction>,
) {
    for entity in &deposits {
        commands
            .entity(entity)
            .insert((MeshMaterial3d(assets.raw.clone()), Transform::default()));
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
        transform.scale = Vec3::ONE;
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
fn responsive_layout(
    windows: Query<&Window>,
    mut previous_size: Local<Option<Vec2>>,
    mut nodes: Query<(
        &mut Node,
        Option<&HudRoot>,
        Option<&ControlBar>,
        Option<&Control>,
        Option<&DesktopDetail>,
    )>,
    mut titles: Query<&mut TextFont, With<HudTitle>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    if *previous_size == Some(size) {
        return;
    }
    *previous_size = Some(size);
    let mobile = mobile_layout(window.width(), window.height());
    for (mut node, hud, bar, control, detail) in &mut nodes {
        if hud.is_some() {
            node.top = px(if mobile { 8 } else { 18 });
            node.left = px(if mobile { 8 } else { 20 });
            node.padding = UiRect::all(px(if mobile { 8 } else { 14 }));
            node.max_width = if mobile {
                px((window.width() - 128.0).max(1.0))
            } else {
                Val::Auto
            };
        }
        if detail.is_some() {
            node.display = if mobile { Display::None } else { Display::Flex };
        }
        if bar.is_some() {
            node.left = px(8);
            node.right = px(8);
            node.justify_content = JustifyContent::Center;
            node.bottom = px(if mobile { 12 } else { 20 });
            node.column_gap = px(if mobile { 6 } else { 8 });
            node.row_gap = px(if mobile { 6 } else { 8 });
        }
        if control.is_some_and(|control| !matches!(control, Control::Hud)) {
            node.width = if mobile {
                px(((window.width() - 34.0) / 4.0).max(44.0))
            } else {
                Val::Auto
            };
            node.padding = UiRect::axes(px(if mobile { 0 } else { 14 }), px(0));
            node.flex_shrink = 0.0;
        }
    }
    for mut font in &mut titles {
        font.font_size = FontSize::Px(if mobile { 18.0 } else { 24.0 });
    }
}
fn button_controls(
    mut buttons: Query<(Ref<Interaction>, &Control, &mut BackgroundColor)>,
    mut state: ResMut<MatchState>,
    mut hud: ResMut<HudVisibility>,
    mut tool: ResMut<TrailTool>,
) {
    // Act only on a new press while updating selected tool colors every frame.
    for (interaction, control, mut color) in &mut buttons {
        if *interaction == Interaction::Pressed && interaction.is_changed() {
            match control {
                Control::Pause => state.paused = !state.paused,
                Control::Speed(speed) => state.speed = *speed,
                Control::Restart => state.restart_requested = true,
                Control::Hud => hud.visible = !hud.visible,
                Control::Tool(selected) => *tool = *selected,
            }
        }
        let selected = matches!(control, Control::Tool(selected) if *selected == *tool);
        color.0 = if selected {
            Color::srgb(0.12, 0.4, 0.48)
        } else if *interaction == Interaction::None {
            Color::srgb(0.13, 0.17, 0.23)
        } else {
            Color::srgb(0.24, 0.32, 0.43)
        };
    }
}
fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut gesture: ResMut<CameraGesture>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<SpectatorCamera>>,
) {
    for (mut transform, mut projection) in &mut cameras {
        transform.translation += Vec3::new(gesture.pan.x, 0.0, gesture.pan.y);
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
            orthographic.scale = (orthographic.scale * gesture.zoom
                + zoom as f32 * time.delta_secs().min(0.1))
            .clamp(0.3, 2.0);
        }
    }
    gesture.pan = Vec2::ZERO;
    gesture.zoom = 1.0;
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
fn browser_mobile_state() {}
#[cfg(target_arch = "wasm32")]
fn browser_mobile_state(
    tool: Res<TrailTool>,
    windows: Query<&Window>,
    cameras: Query<(&Transform, &Projection), With<SpectatorCamera>>,
) {
    let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("game-status"))
    else {
        return;
    };
    let _ = status.set_attribute(
        "data-trail-tool",
        match *tool {
            TrailTool::Draw => "draw",
            TrailTool::Erase => "erase",
        },
    );
    if let Some(window) = windows.iter().next() {
        let _ = status.set_attribute(
            "data-mobile-layout",
            if mobile_layout(window.width(), window.height()) {
                "true"
            } else {
                "false"
            },
        );
    }
    for (transform, projection) in &cameras {
        if let Projection::Orthographic(orthographic) = projection {
            let _ = status.set_attribute("data-camera-scale", &orthographic.scale.to_string());
        }
        let _ = status.set_attribute("data-camera-x", &transform.translation.x.to_string());
        let _ = status.set_attribute("data-camera-z", &transform.translation.z.to_string());
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn browser_state() {}
#[cfg(target_arch = "wasm32")]
fn browser_state(
    state: Res<MatchState>,
    inspector: Res<InspectorState>,
    minions: Query<(), With<Minion>>,
    towers: Query<Option<&PlayerControlled>, With<Tower>>,
    deposits: Query<&RawElement, With<RawElementDeposit>>,
    crown: Query<(), With<DonutVisual>>,
    inventories: Query<(&RawElement, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>,
    trails: Query<(), With<Trail>>,
    economy: Res<RawElementEconomy>,
    drawn: Query<(), With<PlayerDrawnTrail>>,
    stores: Query<&EnergyStore, With<Minion>>,
    clock: Res<EnergyClock>,
    energy_orbs: Query<(), With<EnergyOrbVisual>>,
    projectiles: Query<(), With<EnergyProjectile>>,
    order_visuals: Query<&EntityKind, Or<(With<StackPiece>, With<TowerBeacon>)>>,
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
            (
                "data-player-towers",
                towers
                    .iter()
                    .filter(|controlled| controlled.is_some())
                    .count()
                    .to_string(),
            ),
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
            ("data-raw-initial", economy.initial.to_string()),
            ("data-trails", trails.iter().count().to_string()),
            ("data-drawn-trails", drawn.iter().count().to_string()),
            (
                "data-total-minion-energy",
                stores
                    .iter()
                    .map(|store| u64::from(store.current))
                    .sum::<u64>()
                    .to_string(),
            ),
            ("data-energy-pulses", clock.pulses.to_string()),
            ("data-energy-orbs", energy_orbs.iter().count().to_string()),
            (
                "data-stack-pieces",
                order_visuals
                    .iter()
                    .filter(|kind| **kind == EntityKind::StackVisual)
                    .count()
                    .to_string(),
            ),
            (
                "data-tower-beacons",
                order_visuals
                    .iter()
                    .filter(|kind| **kind == EntityKind::TowerBeacon)
                    .count()
                    .to_string(),
            ),
            (
                "data-energy-projectiles",
                projectiles.iter().count().to_string(),
            ),
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

#[derive(Component)]
struct TowerInfluenceVisual;
/// Each icon is built from UI rectangles so it does not depend on font glyph coverage.
#[derive(Component)]
struct PauseIcon(bool);

fn spawn_pause_icon(button: &mut ChildSpawnerCommands) {
    for paused in [false, true] {
        button
            .spawn((
                Name::new(if paused { "Play icon" } else { "Pause icon" }),
                EntityKind::Ui,
                PauseIcon(paused),
                Node {
                    width: px(18),
                    height: px(20),
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ))
            .with_children(|icon| {
                if paused {
                    for row in 0..10 {
                        let width = (10 - (row * 2_i32 - 9).abs()) as f32 * 1.8;
                        icon.spawn((
                            Name::new("Play icon strip"),
                            EntityKind::Ui,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(2),
                                top: px(row * 2),
                                width: px(width),
                                height: px(2),
                                ..default()
                            },
                            BackgroundColor(Color::WHITE),
                        ));
                    }
                } else {
                    for left in [2, 11] {
                        icon.spawn((
                            Name::new("Pause icon bar"),
                            EntityKind::Ui,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(left),
                                width: px(5),
                                height: px(20),
                                ..default()
                            },
                            BackgroundColor(Color::WHITE),
                        ));
                    }
                }
            });
    }
}

fn sync_control_state(
    state: Res<MatchState>,
    hud: Res<HudVisibility>,
    mut roots: Query<&mut Node, (With<HudRoot>, Without<PauseIcon>)>,
    mut icons: Query<(&PauseIcon, &mut Node), Without<HudRoot>>,
    mut hud_labels: Query<&mut Text, With<HudToggleText>>,
    mut buttons: Query<(
        &Control,
        &mut Name,
        Option<&mut bevy::a11y::AccessibilityNode>,
    )>,
) {
    for mut text in &mut hud_labels {
        text.0 = String::from(if hud.visible { "Hide HUD" } else { "Show HUD" });
    }
    for mut node in &mut roots {
        node.display = if hud.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (icon, mut node) in &mut icons {
        node.display = if icon.0 == state.paused {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (control, mut name, accessible) in &mut buttons {
        match control {
            Control::Pause => *name = Name::new(if state.paused { "Play" } else { "Pause" }),
            Control::Hud => *name = Name::new(if hud.visible { "Hide HUD" } else { "Show HUD" }),
            _ => {}
        }
        if let Some(mut accessible) = accessible {
            accessible.set_label(name.as_str());
        }
    }
}

fn sync_tower_influence_visuals(world: &mut World) {
    let towers: Vec<_> = world
        .query_filtered::<(Entity, &Health, &InfluenceEmitter, &Allegiance), With<Tower>>()
        .iter(world)
        .map(|(entity, health, emitter, allegiance)| {
            (entity, health.current, emitter.radius, allegiance.0)
        })
        .collect();
    for (parent, health, radius, faction) in towers {
        let child = world.get::<Children>(parent).and_then(|children| {
            children
                .iter()
                .find(|child| world.get::<TowerInfluenceVisual>(*child).is_some())
        });
        if health <= 0.0 || !radius.is_finite() || radius <= 0.0 {
            if let Some(child) = child {
                world.despawn(child);
            }
            continue;
        }
        let Some(faction) = world.get::<Faction>(faction) else {
            continue;
        };
        let assets = world.resource::<VisualAssets>();
        let material = assets.influence_materials[faction.index].clone();
        let mesh = assets.influence_ring.clone();
        let transform =
            Transform::from_xyz(0.0, 0.08, 0.0).with_scale(Vec3::new(radius, 0.1, radius));
        if let Some(child) = child {
            world
                .entity_mut(child)
                .insert((transform, MeshMaterial3d(material)));
        } else {
            world.spawn((
                Name::new("Tower recharge area"),
                EntityKind::TowerInfluenceVisual,
                TowerInfluenceVisual,
                ChildOf(parent),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform,
            ));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_control_positions() {}
#[cfg(target_arch = "wasm32")]
fn browser_control_positions(
    controls: Query<(&Control, &ComputedNode, &UiGlobalTransform)>,
    hud: Res<HudVisibility>,
    state: Res<MatchState>,
    areas: Query<(), With<TowerInfluenceVisual>>,
    towers: Query<&InfluenceEmitter, With<Tower>>,
) {
    let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("game-status"))
    else {
        return;
    };
    let mut positions = Vec::new();
    for (control, node, transform) in &controls {
        if node.size() == Vec2::ZERO {
            continue;
        }
        let key = match control {
            Control::Pause => "pause",
            Control::Tool(TrailTool::Draw) => "draw",
            Control::Tool(TrailTool::Erase) => "erase",
            Control::Hud => "hud",
            Control::Restart => "restart",
            Control::Speed(speed) if *speed == 1.0 => "1x",
            Control::Speed(speed) if *speed == 2.0 => "2x",
            Control::Speed(speed) if *speed == 4.0 => "4x",
            _ => continue,
        };
        let center = transform.translation * node.inverse_scale_factor();
        positions.push(format!("\"{key}\":[{},{}]", center.x, center.y));
    }
    let _ = status.set_attribute(
        "data-control-positions",
        &format!("{{{}}}", positions.join(",")),
    );
    let _ = status.set_attribute(
        "data-hud-visible",
        if hud.visible { "true" } else { "false" },
    );
    let _ = status.set_attribute(
        "data-pause-action",
        if state.paused { "play" } else { "pause" },
    );
    let _ = status.set_attribute("data-recharge-areas", &areas.iter().count().to_string());
    let _ = status.set_attribute(
        "data-max-tower-radius",
        &towers
            .iter()
            .map(|emitter| emitter.radius)
            .fold(0.0_f32, f32::max)
            .to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_breakpoint_covers_portrait_and_short_landscape() {
        assert!(mobile_layout(390.0, 844.0));
        assert!(mobile_layout(844.0, 390.0));
        assert!(!mobile_layout(1280.0, 720.0));
        assert!(!mobile_layout(760.0, 540.0));
    }

    #[test]
    fn camera_gesture_is_consumed_once_and_zoom_is_bounded() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Time>()
            .insert_resource(CameraGesture {
                pan: Vec2::new(4.0, -3.0),
                zoom: 10.0,
            })
            .add_systems(Update, camera_controls);
        let camera = app
            .world_mut()
            .spawn((
                SpectatorCamera,
                Transform::default(),
                Projection::from(OrthographicProjection::default_3d()),
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            Vec3::new(4.0, 0.0, -3.0)
        );
        let Projection::Orthographic(projection) = app.world().get::<Projection>(camera).unwrap()
        else {
            panic!("orthographic camera")
        };
        assert_eq!(projection.scale, 2.0);
        app.update();
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            Vec3::new(4.0, 0.0, -3.0)
        );
        assert_eq!(app.world().resource::<CameraGesture>().pan, Vec2::ZERO);
        assert_eq!(app.world().resource::<CameraGesture>().zoom, 1.0);
    }

    #[test]
    fn hud_and_pause_controls_sync_without_hiding_controls_or_resetting_visibility() {
        let mut app = App::new();
        app.init_resource::<MatchState>()
            .init_resource::<HudVisibility>()
            .add_systems(Update, sync_control_state);
        let hud = app.world_mut().spawn((HudRoot, Node::default())).id();
        let button = app
            .world_mut()
            .spawn((Control::Hud, Name::new(""), Node::default()))
            .id();
        let pause = app.world_mut().spawn((Control::Pause, Name::new(""))).id();
        let pause_icon = app
            .world_mut()
            .spawn((PauseIcon(false), Node::default()))
            .id();
        let play_icon = app
            .world_mut()
            .spawn((PauseIcon(true), Node::default()))
            .id();
        app.update();
        assert_eq!(app.world().get::<Node>(hud).unwrap().display, Display::Flex);
        assert_eq!(app.world().get::<Name>(pause).unwrap().as_str(), "Pause");
        assert_eq!(
            app.world().get::<Node>(play_icon).unwrap().display,
            Display::None
        );
        app.world_mut().resource_mut::<HudVisibility>().visible = false;
        app.world_mut().resource_mut::<MatchState>().paused = true;
        app.update();
        assert_eq!(app.world().get::<Node>(hud).unwrap().display, Display::None);
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Name>(button).unwrap().as_str(),
            "Show HUD"
        );
        assert_eq!(app.world().get::<Name>(pause).unwrap().as_str(), "Play");
        assert_eq!(
            app.world().get::<Node>(pause_icon).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(play_icon).unwrap().display,
            Display::Flex
        );
        app.world_mut().insert_resource(MatchState::default());
        app.update();
        assert!(!app.world().resource::<HudVisibility>().visible);
    }

    #[test]
    fn selecting_erase_persists_after_button_release() {
        let mut app = App::new();
        app.init_resource::<MatchState>()
            .init_resource::<HudVisibility>()
            .init_resource::<TrailTool>()
            .add_systems(Update, button_controls);
        let erase = app
            .world_mut()
            .spawn((
                Interaction::Pressed,
                Control::Tool(TrailTool::Erase),
                BackgroundColor::default(),
            ))
            .id();
        app.update();
        assert_eq!(*app.world().resource::<TrailTool>(), TrailTool::Erase);
        app.world_mut().entity_mut(erase).insert(Interaction::None);
        app.update();
        assert_eq!(*app.world().resource::<TrailTool>(), TrailTool::Erase);
        assert_eq!(
            app.world().get::<BackgroundColor>(erase).unwrap().0,
            Color::srgb(0.12, 0.4, 0.48)
        );
    }

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
            assert!(first.translation.y > if tower { 0.0 } else { 0.77 });
            assert_ne!(first.translation, donut_transform(1, tower).translation);
        }
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
            influence_ring: meshes.add(Torus::default()),
            influence_materials: palette.clone(),
            energy: meshes.add(Sphere::new(0.2)),
            raw: raw.clone(),
            trail_materials: fades.clone(),
            materials: palette.clone(),
        });
        app.init_resource::<SimulationConfig>();
        app.add_systems(
            Update,
            (
                attach_visuals,
                sync_stacks,
                sync_donut_visuals,
                sync_energy_orbs,
                attach_projectile_visuals,
                sync_projectile_visuals,
                sync_visuals,
                attach_resource_visuals,
                sync_gathering_visuals,
                sync_tower_influence_visuals,
            )
                .chain(),
        );
        let own = app.world_mut().spawn(Faction { index: 0 }).id();
        let enemy = app.world_mut().spawn(Faction { index: 1 }).id();
        let minion = app
            .world_mut()
            .spawn((
                Minion,
                Position(Vec2::ZERO),
                Allegiance(own),
                RawElement(0),
                EnergyStore {
                    current: 3,
                    capacity: 5,
                },
            ))
            .id();
        let tower = app
            .world_mut()
            .spawn((
                Tower { slot: 0 },
                InfluenceEmitter {
                    radius: 12.0,
                    strength: 1.0,
                },
                Health {
                    current: 19.0,
                    max: 20.0,
                },
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
        let drawn = app
            .world_mut()
            .spawn((
                PlayerDrawnTrail { serial: 1 },
                Trail {
                    trip: 1,
                    sequence: 0,
                    remaining: 15.0,
                },
                Position(Vec2::Y),
                Allegiance(own),
            ))
            .id();
        let projectile = app
            .world_mut()
            .spawn((
                Name::new("Energy projectile"),
                EntityKind::EnergyProjectile,
                MatchEntity,
                EnergyProjectile {
                    target: minion,
                    damage: 1.0,
                    speed: 10.0,
                    remaining: 5.0,
                },
                Allegiance(enemy),
                Position(Vec2::new(4.0, 8.0)),
            ))
            .id();
        app.world_mut().run_schedule(Update);
        let pieces = |world: &mut World, parent: Entity| -> Vec<Entity> {
            world
                .query_filtered::<(Entity, &ChildOf), With<StackPiece>>()
                .iter(world)
                .filter_map(|(entity, child)| (child.parent() == parent).then_some(entity))
                .collect()
        };
        assert_eq!(pieces(app.world_mut(), tower).len(), 19);
        assert_eq!(pieces(app.world_mut(), deposit).len(), 5);
        let area = app
            .world()
            .get::<Children>(tower)
            .unwrap()
            .iter()
            .find(|child| app.world().get::<TowerInfluenceVisual>(*child).is_some())
            .unwrap();
        assert_eq!(app.world().get::<Transform>(area).unwrap().scale.x, 12.0);
        app.world_mut()
            .get_mut::<InfluenceEmitter>(tower)
            .unwrap()
            .radius = 18.0;
        app.world_mut().run_schedule(Update);
        assert_eq!(app.world().get::<Transform>(area).unwrap().scale.z, 18.0);
        assert_eq!(
            *app.world().get::<EntityKind>(area).unwrap(),
            EntityKind::TowerInfluenceVisual
        );
        let stack = *app.world().get::<Stack>(tower).unwrap();
        assert_eq!(stack.count, 19);
        assert!(stack.transform(16).translation.y > stack.transform(0).translation.y);
        app.world_mut().get_mut::<Health>(tower).unwrap().current = 1.0;
        app.world_mut().get_mut::<RawElement>(deposit).unwrap().0 = 2;
        app.world_mut().run_schedule(Update);
        assert_eq!(pieces(app.world_mut(), tower).len(), 1);
        let deposit_pieces = pieces(app.world_mut(), deposit);
        assert_eq!(deposit_pieces.len(), 2);
        assert_eq!(
            app.world()
                .get::<Transform>(projectile)
                .unwrap()
                .translation,
            Vec3::new(4.0, 1.0, 8.0)
        );
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(projectile)
                .unwrap()
                .0,
            palette[1]
        );
        assert!(app.world().get::<ChildOf>(projectile).is_none());
        app.world_mut().get_mut::<Position>(projectile).unwrap().0 = Vec2::new(9.0, 2.0);
        app.world_mut().run_schedule(Update);
        assert_eq!(
            app.world()
                .get::<Transform>(projectile)
                .unwrap()
                .translation,
            Vec3::new(9.0, 1.0, 2.0)
        );
        app.world_mut().despawn(projectile);
        let orbs = |world: &mut World| -> Vec<Entity> {
            world
                .query_filtered::<Entity, With<EnergyOrbVisual>>()
                .iter(world)
                .collect()
        };
        let initial_orbs = orbs(app.world_mut());
        assert_eq!(initial_orbs.len(), 3);
        for orb in &initial_orbs {
            assert_eq!(
                *app.world().get::<EntityKind>(*orb).unwrap(),
                EntityKind::EnergyOrbVisual
            );
            assert_eq!(app.world().get::<ChildOf>(*orb).unwrap().parent(), minion);
            assert!(app.world().get::<Name>(*orb).is_some());
        }
        app.world_mut()
            .get_mut::<EnergyStore>(minion)
            .unwrap()
            .current = 1;
        app.world_mut().run_schedule(Update);
        assert_eq!(orbs(app.world_mut()).len(), 1);
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
        assert_eq!(
            app.world().get::<Mesh3d>(drawn).unwrap().0,
            app.world().get::<Mesh3d>(trail).unwrap().0
        );
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(drawn)
                .unwrap()
                .0,
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(trail)
                .unwrap()
                .0
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
        let remaining_orbs = orbs(app.world_mut());
        app.world_mut().despawn(minion);
        assert!(
            remaining_orbs
                .iter()
                .all(|orb| app.world().get_entity(*orb).is_err())
        );
        app.world_mut().run_schedule(Update);
        assert!(
            carried
                .iter()
                .all(|child| app.world().get_entity(*child).is_err())
        );
        app.world_mut().despawn(deposit);
        assert!(
            deposit_pieces
                .iter()
                .all(|piece| app.world().get_entity(*piece).is_err())
        );
        let tower_pieces = pieces(app.world_mut(), tower);
        let beacon = app
            .world()
            .get::<Children>(tower)
            .unwrap()
            .iter()
            .find(|child| app.world().get::<TowerBeacon>(*child).is_some())
            .unwrap();
        let remaining = donuts(app.world_mut(), tower);
        app.world_mut().despawn(tower);
        assert!(
            tower_pieces
                .iter()
                .all(|piece| app.world().get_entity(*piece).is_err())
        );
        assert!(app.world().get_entity(beacon).is_err());
        assert!(app.world().get_entity(area).is_err());
        assert!(
            remaining
                .iter()
                .all(|child| app.world().get_entity(*child).is_err())
        );
    }
}
