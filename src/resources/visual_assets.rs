use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct VisualAssets {
    pub(crate) tower: Handle<Mesh>,
    pub(crate) minion: Handle<Mesh>,
    pub(crate) ground: Handle<Mesh>,
    pub(crate) deposit: Handle<Mesh>,
    pub(crate) trail: Handle<Mesh>,
    pub(crate) donut: Handle<Mesh>,
    pub(crate) influence_ring: Handle<Mesh>,
    pub(crate) influence_materials: Vec<Handle<StandardMaterial>>,
    pub(crate) sphere: Handle<Mesh>,
    pub(crate) elements: [Handle<StandardMaterial>; 5],
    pub(crate) trail_materials: Vec<Vec<Handle<StandardMaterial>>>,
    pub(crate) minion_health_materials: Vec<Vec<Handle<StandardMaterial>>>,
    pub(crate) materials: Vec<Handle<StandardMaterial>>,
}
