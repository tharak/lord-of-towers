use crate::types::map_settings::MapSettings;
use crate::types::map_ui::MapPhase;
use bevy::prelude::*;
#[derive(Resource)]
pub(crate) struct MapCreation {
    pub settings: MapSettings,
    /// Temporarily keep the map editor out of the player flow.
    pub options_enabled: bool,
    pub phase: MapPhase,
    pub show_settings: bool,
    pub pending_frames: u8,
    pub message: String,
}
impl Default for MapCreation {
    fn default() -> Self {
        let options_enabled = false;
        Self {
            settings: MapSettings::default(),
            options_enabled,
            phase: if options_enabled {
                MapPhase::Configuring
            } else {
                MapPhase::Generating
            },
            show_settings: options_enabled,
            pending_frames: 0,
            message: if options_enabled {
                "Choose your map, then generate a preview."
            } else {
                "Preparing the battlefield…"
            }
            .into(),
        }
    }
}
