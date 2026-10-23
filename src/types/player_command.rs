use crate::types::Element;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub enum PlayerCommand {
    Equip {
        minion: Entity,
        tower: Entity,
        slot: usize,
        element: Element,
    },
    ReturnEquipment {
        minion: Entity,
        tower: Entity,
        slot: usize,
    },
    DeliverCargo {
        minion: Entity,
        tower: Entity,
    },
    CancelOrder {
        minion: Entity,
    },
    CreateMinion {
        tower: Entity,
        counts: [u32; 5],
    },
}
