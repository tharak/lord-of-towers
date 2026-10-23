use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct TowerAlert {
    pub tower: Entity,
    pub attacker: Entity,
}
