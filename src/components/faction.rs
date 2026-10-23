//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Faction {
    pub index: usize,
}
