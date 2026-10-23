use crate::components::{
    Allegiance, ElementStats, Health, InfluenceEmitter, Minion, Position, Tower,
};
use crate::resources::MatchState;
use crate::scheduling::simulation_delta;
use bevy::prelude::*;

/// Friendly base circles heal once per tick, independent of territory sampling.
pub(crate) fn heal_minions(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    towers: Query<
        (&Position, &Allegiance, &InfluenceEmitter, &Health),
        (With<Tower>, Without<Minion>),
    >,
    mut minions: Query<
        (&Position, &Allegiance, &ElementStats, &mut Health),
        (With<Minion>, Without<Tower>),
    >,
) {
    let dt = simulation_delta(&time, &state);
    for (position, allegiance, stats, mut health) in &mut minions {
        if health.current <= 0.0 || health.current >= health.max || stats.earth == 0 {
            continue;
        }
        let inside_friendly_base =
            towers
                .iter()
                .any(|(base_position, owner, emitter, base_health)| {
                    owner == allegiance
                        && base_health.current > 0.0
                        && emitter.radius.is_finite()
                        && emitter.radius > 0.0
                        && position.0.distance_squared(base_position.0)
                            <= emitter.radius * emitter.radius
                });
        if inside_friendly_base {
            health.current = (health.current + stats.earth as f32 * dt).min(health.max);
        }
    }
}
