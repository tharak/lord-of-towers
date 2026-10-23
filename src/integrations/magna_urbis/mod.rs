//! The only application layer allowed to call Magna Urbis.
//! Inputs and outputs belong to Lord of Towers; upstream types never escape.
mod decoration;
mod settlements;
mod terrain;
use crate::types::{map_data::MapData, map_settings::MapGenerationRequest};

/// Automatic startup retains the default scenery, with a simpler upstream path on error.
pub(crate) fn generate_default(request: MapGenerationRequest) -> Result<MapData, String> {
    generate(request).or_else(|error| {
        bevy::log::warn!("Default map generation failed; trying simple terrain: {error}");
        use crate::types::map_settings::{MapDensity, MapRelief, MapRoads, MapSettings, MapWater};
        generate(MapGenerationRequest {
            settings: MapSettings {
                relief: MapRelief::Plains,
                water: MapWater::None,
                settlements: MapDensity::None,
                roads: MapRoads::None,
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
    Ok(decoration::convert(
        surface,
        roads,
        buildings,
        settlement_count,
        &request,
    ))
}
