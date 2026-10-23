//! Constructs tower entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{
    Allegiance, ElementInventory, EntityKind, Health, InfluenceEmitter, MatchEntity,
    PlayerControlled, Position, SimulationId, Tower,
};
use crate::resources::SimulationConfig;
use bevy::prelude::*;

pub(crate) fn spawn_tower(
    commands: &mut Commands,
    config: &SimulationConfig,
    index: usize,
    name: &'static str,
    faction: Entity,
    position: Vec2,
) -> Entity {
    let tower = commands
        .spawn((
            Name::new(format!("{name} tower")),
            EntityKind::Tower,
            MatchEntity,
            Tower { slot: index },
            SimulationId(index as u64),
            Allegiance(faction),
            Position(position),
            Health {
                current: config.tower_health,
                max: config.tower_health,
            },
            ElementInventory::default(),
            InfluenceEmitter {
                radius: config.tower_influence_radius,
                strength: 4.0,
            },
        ))
        .id();
    if index == 0 {
        commands.entity(tower).insert(PlayerControlled);
    }
    tower
}
