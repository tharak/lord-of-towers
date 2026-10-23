use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct InspectorFilter {
    pub(crate) kind: usize,
    pub(crate) faction: usize,
    pub(crate) engine: bool,
    pub(crate) page: usize,
    pub(crate) component_page: usize,
    pub(crate) next_refresh: f64,
}
