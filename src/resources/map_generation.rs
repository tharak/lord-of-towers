use crate::types::map_data::MapData;
use bevy::{prelude::*, tasks::Task};
#[derive(Resource, Default)]
pub(crate) struct MapGeneration(pub Option<Task<Result<MapData, String>>>);
