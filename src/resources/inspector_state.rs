use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct InspectorState {
    pub open: bool,
}
