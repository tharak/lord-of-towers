//! Resource initialization and system ordering for this subsystem.
use crate::resources::{CameraGesture, HudVisibility};
use crate::systems::presentation::browser::{
    browser_control_positions, browser_mobile_state, browser_state, browser_visibility,
};
use crate::systems::presentation::camera::camera_controls;
use crate::systems::presentation::controls::{
    button_controls, keyboard_controls, sync_control_state,
};
use crate::systems::presentation::equipment::sync_equipped_visuals;
use crate::systems::presentation::gathering::{attach_resource_visuals, sync_gathering_visuals};
use crate::systems::presentation::ground::ground_colors;
use crate::systems::presentation::hud::update_hud;
use crate::systems::presentation::influence::sync_tower_influence_visuals;
use crate::systems::presentation::inventory::sync_donut_visuals;
use crate::systems::presentation::layout::responsive_layout;
use crate::systems::presentation::production::sync_production_indicators;
use crate::systems::presentation::projectiles::{
    attach_projectile_visuals, sync_projectile_visuals,
};
use crate::systems::presentation::setup::setup_presentation;
use crate::systems::presentation::stacks::sync_stacks;
use crate::systems::presentation::units::{attach_visuals, sync_visuals};
use bevy::prelude::*;

pub struct PresentationPlugin;
impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
            .init_resource::<HudVisibility>()
            .init_resource::<CameraGesture>()
            .add_systems(Startup, setup_presentation)
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
                    sync_equipped_visuals,
                    sync_production_indicators,
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
