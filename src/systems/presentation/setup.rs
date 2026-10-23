use crate::entities::battlefield::spawn_battlefield;
use crate::entities::camera::spawn_camera;
use crate::entities::light::spawn_light;
use crate::entities::ui::hud::spawn_hud;
use crate::resources::{SimulationConfig, VisualAssets};
use crate::types::{Element, FACTION_RGB, HEALTH_PERCENT_STEPS, TRAIL_BUCKETS, faction_color};
use bevy::prelude::*;

pub(crate) fn setup_presentation(
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
    spawn_battlefield(&mut commands, ground.clone(), &mut materials);
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
    // Shared shades keep health presentation independent of minion lifetimes.
    let minion_health_materials = FACTION_RGB
        .iter()
        .map(|rgb| {
            (0..=HEALTH_PERCENT_STEPS)
                .map(|step| {
                    let fraction = step as f32 / HEALTH_PERCENT_STEPS as f32;
                    let [r, g, b] =
                        rgb.map(|channel| (89.0 + (channel as f32 - 89.0) * fraction) / 255.0);
                    materials.add(StandardMaterial {
                        base_color: Color::srgb(r, g, b),
                        perceptual_roughness: 0.8,
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
        sphere: meshes.add(Sphere::new(0.2)),
        elements: Element::ALL.map(|element| {
            materials.add(StandardMaterial {
                base_color: element.color(),
                perceptual_roughness: 0.45,
                unlit: element != Element::Void,
                ..default()
            })
        }),
        trail_materials,
        minion_health_materials,
        materials: palette,
    });
    spawn_light(&mut commands);
    spawn_camera(&mut commands);
    spawn_hud(&mut commands);
}
