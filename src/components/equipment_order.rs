//! ECS component data; behavior lives in systems.
use crate::types::TowerOrderAction;
use bevy::prelude::*;

/// Reservation lives in this entity's ElementInventory, never in spendable tower stock.
#[derive(Component, Clone, Copy, Debug)]
pub struct EquipmentOrder {
    pub minion: Entity,
    pub tower: Entity,
    pub source_position: Vec2,
    pub action: TowerOrderAction,
}
