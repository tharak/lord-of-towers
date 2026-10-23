use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerSelection {
    pub entity: Option<Entity>,
    pub slot: usize,
    pub recipe: [u32; 5],
}
