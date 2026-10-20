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
                    browser_state,
                    attach_visuals,
                    sync_visuals,
                    ground_colors,
                    update_hud,
                    camera_controls,
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
    materials: Vec<Handle<StandardMaterial>>,
}
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
    commands.insert_resource(VisualAssets {
        tower,
        minion,
        ground,
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
                Text::new("Starting simulation…"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
            ));
            for i in 0..5 {
                parent.spawn((
                    Name::new(format!("{} statistics", FACTION_NAMES[i])),
                    EntityKind::Ui,
                    FactionText(i),
                    Text::new(FACTION_NAMES[i]),
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
                    "Pan: WASD / arrows   Zoom: + / −\nSpace: pause   1/2/3: speed   R: restart",
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
                    option_env!("BUILD_COMMIT").unwrap_or("development")
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
                ("1×", Control::Speed(1.0)),
                ("2×", Control::Speed(2.0)),
                ("4×", Control::Speed(4.0)),
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
fn sync_visuals(
    assets: Res<VisualAssets>,
    factions: Query<&Faction>,
    mut entities: Query<(
        &Position,
        &Allegiance,
        Option<&Tower>,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
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
fn ground_colors(
    field: Res<InfluenceField>,
    assets: Res<VisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut revision: Local<Option<u64>>,
) {
    if *revision == Some(field.revision) {
        return;
    }
    let Some(mesh) = meshes.get_mut(&assets.ground) else {
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
    units: Query<(&Allegiance, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>,
    mut texts: Query<(&mut Text, Option<&FactionText>), Or<(With<StatusText>, With<FactionText>)>>,
) {
    let mut towers = [0; 5];
    let mut minions = [0; 5];
    for (allegiance, tower) in &units {
        if let Ok(faction) = factions.get(allegiance.0) {
            if tower.is_some() {
                towers[faction.index] += 1;
            } else {
                minions[faction.index] += 1;
            }
        }
    }
    let territory = field.territory_percentages();
    for (mut text, faction) in &mut texts {
        text.0 = if let Some(FactionText(i)) = faction {
            format!(
                "{}  |  {} towers · {} minions · {:.0}%",
                FACTION_NAMES[*i], towers[*i], minions[*i], territory[*i]
            )
        } else if let Some(winner) = state.winner {
            format!("{} wins! Restart to play again.", FACTION_NAMES[winner])
        } else {
            format!(
                "{} · {:.0}× speed · {:.0}s",
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
