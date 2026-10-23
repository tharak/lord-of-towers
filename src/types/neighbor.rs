use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct Neighbor {
    pub(crate) entity: Entity,
    pub(crate) position: Vec2,
    pub(crate) faction: Entity,
    pub(crate) tower: bool,
    pub(crate) stable_id: u64,
}
