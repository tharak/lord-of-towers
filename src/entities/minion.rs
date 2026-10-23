//! Constructs minion entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{
    Allegiance, ArmState, Attack, ElementInventory, ElementStats, EntityKind, EquippedElements,
    GatheringState, Health, InfluenceEmitter, LegState, MatchEntity, Minion, Movement, OriginTower,
    Position, ShootingPause, SimulationId, Vision,
};
use crate::resources::{RandomStream, SimulationConfig, SpawnSequence};
use bevy::prelude::*;

pub(crate) fn spawn_minion(
    commands: &mut Commands,
    config: &SimulationConfig,
    random: &mut RandomStream,
    sequence: &mut SpawnSequence,
    tower: Entity,
    position: Vec2,
    allegiance: Allegiance,
    stats: ElementStats,
) {
    let direction = random.direction();
    let location = (position + direction * 2.8).clamp(
        Vec2::splat(-config.half_extent),
        Vec2::splat(config.half_extent),
    );
    let stable_id = sequence.0;
    sequence.0 += 1;
    let health = config.minion_health;
    commands.spawn((
        (
            SimulationId(stable_id),
            stats,
            EquippedElements::default(),
            ElementInventory::default(),
            Vision {
                radius: config.vision_radius,
            },
            GatheringState::Exploring,
            ShootingPause::default(),
            LegState::default(),
            ArmState::default(),
        ),
        (
            Name::new("Minion"),
            EntityKind::Minion,
            MatchEntity,
            Minion,
            allegiance,
            OriginTower(tower),
            Position(location),
            Health {
                current: health,
                max: health,
            },
            Movement {
                speed: config.minion_speed,
                target: location + direction,
                retarget_in: 0.0,
            },
            Attack {
                damage: 10.0,
                range: 6.0,
                cooldown: 0.8,
                remaining: 0.0,
            },
            InfluenceEmitter {
                radius: 5.0,
                strength: 1.0,
            },
        ),
    ));
}
