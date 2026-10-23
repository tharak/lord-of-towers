use crate::types::map_settings::MapSettings;
use crate::types::map_ui::MapPhase;
use bevy::prelude::*;
#[derive(Resource)]
pub(crate) struct MapCreation {
    pub settings: MapSettings,
    pub phase: MapPhase,
    pub show_settings: bool,
    pub pending_frames: u8,
    pub message: String,
}
impl Default for MapCreation {
    fn default() -> Self {
        Self {
            settings: MapSettings::default(),
            phase: MapPhase::Configuring,
            show_settings: true,
            pending_frames: 0,
            message: "Choose your map, then generate a preview.".into(),
        }
    }
}
