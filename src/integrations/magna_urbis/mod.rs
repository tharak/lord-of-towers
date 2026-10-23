//! The only application layer allowed to call Magna Urbis.
//! Inputs and outputs belong to Lord of Towers; upstream types never escape.
mod decoration;
mod settlements;
mod terrain;
use crate::types::{map_data::MapData, map_settings::MapGenerationRequest};

/// Automatic startup retries simpler generation while preserving a requested city.
pub(crate) fn generate_default(request: MapGenerationRequest) -> Result<MapData, String> {
    generate(request).or_else(|error| {
        bevy::log::warn!("Default map generation failed; retrying without water: {error}");
        use crate::types::map_settings::{
            MapDensity, MapPreset, MapRelief, MapRoads, MapSettings, MapWater,
        };
        let city = request.settings.preset == MapPreset::Town;
        generate(MapGenerationRequest {
            settings: MapSettings {
                relief: MapRelief::Plains,
                water: MapWater::None,
                settlements: if city {
                    request.settings.settlements
                } else {
                    MapDensity::None
                },
                roads: if city {
                    request.settings.roads
                } else {
                    MapRoads::None
                },
                ..request.settings
            },
            ..request
        })
    })
}

pub(crate) fn generate(request: MapGenerationRequest) -> Result<MapData, String> {
    if !request.half_extent.is_finite() || request.half_extent < 10.0 {
        return Err("The battlefield size is invalid.".into());
    }
    let surface = terrain::generate(&request.settings)?;
    let (roads, buildings, settlement_count) = settlements::generate(&surface, &request)?;
    if request.settings.preset == crate::types::map_settings::MapPreset::Town
        && request.settings.settlements != crate::types::map_settings::MapDensity::None
        && buildings.is_empty()
    {
        return Err("The city generator produced no buildings".into());
    }
    Ok(decoration::convert(
        surface,
        roads,
        buildings,
        settlement_count,
        &request,
    ))
}
