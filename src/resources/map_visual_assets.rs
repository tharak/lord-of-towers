use bevy::prelude::*;
#[derive(Resource)]
pub(crate) struct MapVisualAssets {
    pub cube: Handle<Mesh>,
    pub canopy: Handle<Mesh>,
    pub rock: Handle<Mesh>,
    pub walls: Handle<StandardMaterial>,
    pub roof: Handle<StandardMaterial>,
    pub trunk: Handle<StandardMaterial>,
    pub leaves: [Handle<StandardMaterial>; 3],
    pub stone: Handle<StandardMaterial>,
}
