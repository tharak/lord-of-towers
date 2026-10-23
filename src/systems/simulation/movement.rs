use crate::components::{DragControl, LegState, Minion, Movement, Position, ShootingPause};
use crate::resources::{MatchState, SimulationConfig, SpatialIndex};
use crate::scheduling::simulation_delta;
use crate::types::DragPhase;
use bevy::prelude::*;

/// Remove completed movement intents immediately so targeting can resume this tick.
pub(crate) fn prepare_drag(world: &mut World) {
    let half_extent = world.resource::<SimulationConfig>().half_extent;
    let completed: Vec<Entity> = world
        .query_filtered::<(Entity, &Position, &DragControl), With<Minion>>()
        .iter(world)
        .filter(|(_, position, drag)| {
            drag.phase == DragPhase::Released
                && position.0.distance(
                    drag.target
                        .clamp(Vec2::splat(-half_extent), Vec2::splat(half_extent)),
                ) <= 1.0
        })
        .map(|(entity, _, _)| entity)
        .collect();
    for entity in completed {
        let mut minion = world.entity_mut(entity);
        minion.remove::<DragControl>();
        if let Some(mut movement) = minion.get_mut::<Movement>() {
            movement.retarget_in = 0.0;
        }
        minion.insert(LegState::Idle);
    }
}
pub(crate) fn move_minions(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    config: Res<SimulationConfig>,
    spatial: Res<SpatialIndex>,
    mut minions: Query<
        (
            Entity,
            &mut Position,
            &Movement,
            Option<&mut ShootingPause>,
            Option<&DragControl>,
            &mut LegState,
        ),
        With<Minion>,
    >,
) {
    let dt = simulation_delta(&time, &state);
    for (entity, mut position, movement, pause, drag, mut legs) in &mut minions {
        if let Some(drag) = drag {
            let target = drag.target.clamp(
                Vec2::splat(-config.half_extent),
                Vec2::splat(config.half_extent),
            );
            let offset = target - position.0;
            position.0 += offset.normalize_or_zero() * (movement.speed * dt).min(offset.length());
            *legs = match drag.phase {
                DragPhase::Held => LegState::DragStart,
                DragPhase::Released => LegState::DragRelease,
            };
            continue;
        }
        if let Some(mut pause) = pause
            && pause.0 > 0.0
        {
            pause.0 = (pause.0 - dt).max(0.0);
            *legs = LegState::Idle;
            continue;
        }
        let previous = position.0;
        let offset = movement.target - position.0;
        let mut velocity =
            offset.normalize_or_zero() * movement.speed.min(offset.length() / dt.max(0.0001));
        for other in spatial
            .nearby(position.0, 1.1)
            .filter(|entry| entry.entity != entity && !entry.tower)
        {
            let away = position.0 - other.position;
            let distance = away.length();
            if distance > 0.001 {
                velocity += away / distance * (1.1 - distance) * 3.0;
            }
        }
        position.0 = (position.0 + velocity.clamp_length_max(movement.speed * 1.25) * dt).clamp(
            Vec2::splat(-config.half_extent),
            Vec2::splat(config.half_extent),
        );
        *legs = if position.0 != previous {
            LegState::Moving
        } else {
            LegState::Idle
        };
    }
}
