use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ElementEconomy {
    pub initial: [u32; 5],
    pub consumed: [u32; 5],
}
