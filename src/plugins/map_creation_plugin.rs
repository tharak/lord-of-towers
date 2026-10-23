//! Pre-match settings and preview orchestration. Generator access stays in the adapter.
use crate::{
    entities::ui::map_setup::spawn_map_setup,
    resources::{MapCreation, MapGeneration, MapLandscape, MatchState},
    systems::{
        map_creation::{
            controls::map_actions,
            generation::generate_preview,
            scenery::{setup_map_assets, update_map_scene},
            ui::{scroll_map_settings, update_map_ui},
        },
        presentation::{
            controls::{button_controls, sync_control_state},
            layout::responsive_layout,
        },
    },
};
use bevy::prelude::*;
pub struct MapCreationPlugin;
impl Plugin for MapCreationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapCreation>()
            .init_resource::<MapGeneration>()
            .init_resource::<MapLandscape>()
            .init_resource::<MatchState>()
            .add_systems(Startup, (setup_map_assets, spawn_map_setup))
            .add_systems(
                Update,
                (
                    map_actions,
                    generate_preview,
                    update_map_scene,
                    update_map_ui,
                    scroll_map_settings,
                )
                    .chain()
                    .after(button_controls)
                    .after(sync_control_state)
                    .after(responsive_layout),
            );
        app.world_mut().resource_mut::<MatchState>().map_ready = false;
    }
}
