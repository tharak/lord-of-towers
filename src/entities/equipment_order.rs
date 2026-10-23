//! Constructs equipment order entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{ElementInventory, EntityKind, EquipmentOrder, MatchEntity};
use crate::types::TowerOrderAction;
use bevy::prelude::*;

pub(crate) fn spawn_equipment_order(
    world: &mut World,
    minion: Entity,
    tower: Entity,
    source_position: Vec2,
    action: TowerOrderAction,
    reserved: ElementInventory,
) -> Entity {
    world
        .spawn((
            Name::new("Equipment order"),
            MatchEntity,
            EntityKind::EquipmentOrder,
            EquipmentOrder {
                minion,
                tower,
                source_position,
                action,
            },
            reserved,
        ))
        .id()
}
