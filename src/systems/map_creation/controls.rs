use crate::components::MapSetupAction;
use crate::resources::{MapCreation, MapLandscape, MatchState, SimulationConfig};
use crate::types::{map_settings::*, map_ui::MapPhase};
use bevy::prelude::*;
fn density(value: MapDensity) -> MapDensity {
    match value {
        MapDensity::None => MapDensity::Sparse,
        MapDensity::Sparse => MapDensity::Dense,
        MapDensity::Dense => MapDensity::None,
    }
}
pub(crate) fn map_actions(
    buttons: Query<(Ref<Interaction>, &MapSetupAction)>,
    mut map: ResMut<MapCreation>,
    landscape: Res<MapLandscape>,
    mut state: ResMut<MatchState>,
    mut config: ResMut<SimulationConfig>,
) {
    for (interaction, action) in &buttons {
        if !interaction.is_changed() || *interaction != Interaction::Pressed {
            continue;
        }
        if matches!(action, MapSetupAction::TogglePanel) {
            map.show_settings = !map.show_settings;
            continue;
        }
        if matches!(action, MapSetupAction::NewMap) {
            if map.phase == MapPhase::Playing {
                map.phase = MapPhase::Configuring;
                map.message = "Choose settings for a new match.".into();
                state.restart_requested = true;
            }
            map.show_settings = !map.show_settings || map.phase == MapPhase::Configuring;
            continue;
        }
        if matches!(map.phase, MapPhase::Generating | MapPhase::Playing) {
            continue;
        }
        match action {
            MapSetupAction::Generate => {
                map.phase = MapPhase::Generating;
                map.pending_frames = 0;
                map.message = "Generating terrain, water, roads and settlements…".into();
            }
            MapSetupAction::Start => {
                if map.phase == MapPhase::Ready && landscape.data.is_some() {
                    config.seed = map.settings.seed;
                    map.phase = MapPhase::Playing;
                    map.show_settings = false;
                    state.restart_requested = true;
                }
            }
            _ => {
                let settings = &mut map.settings;
                match action {
                    MapSetupAction::Preset => {
                        let next = match settings.preset {
                            MapPreset::Landscape => MapPreset::Town,
                            MapPreset::Town => MapPreset::Terrain,
                            MapPreset::Terrain => MapPreset::Landscape,
                        };
                        settings.apply_preset(next);
                    }
                    MapSetupAction::Relief => {
                        settings.relief = match settings.relief {
                            MapRelief::Plains => MapRelief::Hills,
                            MapRelief::Hills => MapRelief::Valley,
                            MapRelief::Valley => MapRelief::Mountains,
                            MapRelief::Mountains => MapRelief::Plains,
                        }
                    }
                    MapSetupAction::Water => {
                        settings.water = match settings.water {
                            MapWater::None => MapWater::Rivers,
                            MapWater::Rivers => MapWater::Lakes,
                            MapWater::Lakes => MapWater::RiversAndLakes,
                            MapWater::RiversAndLakes => MapWater::None,
                        }
                    }
                    MapSetupAction::Vegetation => {
                        settings.vegetation = density(settings.vegetation)
                    }
                    MapSetupAction::Settlements => {
                        settings.settlements = density(settings.settlements)
                    }
                    MapSetupAction::Roads => {
                        settings.roads = match settings.roads {
                            MapRoads::None => MapRoads::Regional,
                            MapRoads::Regional => MapRoads::Streets,
                            MapRoads::Streets => MapRoads::None,
                        }
                    }
                    MapSetupAction::Climate => {
                        settings.climate = match settings.climate {
                            MapClimate::Temperate => MapClimate::Arid,
                            MapClimate::Arid => MapClimate::Snow,
                            MapClimate::Snow => MapClimate::Temperate,
                        }
                    }
                    MapSetupAction::SeedPrevious => settings.seed = settings.seed.wrapping_sub(1),
                    MapSetupAction::SeedNext => settings.seed = settings.seed.wrapping_add(1),
                    MapSetupAction::SeedShuffle => {
                        settings.seed = settings
                            .seed
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407)
                    }
                    _ => {}
                }
                map.phase = MapPhase::Configuring;
                map.message = "Settings changed. Generate a new preview to play.".into();
            }
        }
    }
}
