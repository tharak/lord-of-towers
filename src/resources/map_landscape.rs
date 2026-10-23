use crate::types::map_data::MapData;
use bevy::prelude::*;
#[derive(Resource, Default)]
pub(crate) struct MapLandscape {
    pub data: Option<MapData>,
    pub revision: u64,
}
