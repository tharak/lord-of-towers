//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Faction,
    Tower,
    Minion,
    ElementDeposit,
    Trail,
    DonutVisual,
    EquippedElementVisual,
    Projectile,
    StackVisual,
    TowerBeacon,
    TowerInfluenceVisual,
    EquipmentOrder,
    ProductionIndicator,
    Battlefield,
    Camera,
    Light,
    Ui,
}
