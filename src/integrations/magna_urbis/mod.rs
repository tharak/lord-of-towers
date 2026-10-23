//! The only application layer allowed to call Magna Urbis.
//! Inputs and outputs belong to Lord of Towers; upstream types never escape.
mod decoration;
mod settlements;
mod terrain;
use crate::types::{map_data::MapData, map_settings::MapGenerationRequest};

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
