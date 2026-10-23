use crate::components::{Allegiance, Faction, InfluenceEmitter, Position};
use crate::resources::{InfluenceField, MatchState, SimulationConfig};
use crate::scheduling::simulation_delta;
use crate::types::INTERVAL;
use bevy::prelude::*;

pub(crate) fn update_influence(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    config: Res<SimulationConfig>,
    factions: Query<&Faction>,
    emitters: Query<(&Position, &Allegiance, &InfluenceEmitter)>,
    mut field: ResMut<InfluenceField>,
) {
    field.elapsed += simulation_delta(&time, &state);
    if field.elapsed + f32::EPSILON < INTERVAL {
        return;
    }
    field.elapsed %= INTERVAL;
    field.half_extent = config.half_extent;
    let mut sources: Vec<_> = emitters
        .iter()
        .filter_map(|(position, allegiance, emitter)| {
            factions
                .get(allegiance.0)
                .ok()
                .map(|faction| (position.0, faction.index, emitter.radius, emitter.strength))
        })
        .collect();
    sources.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then(a.0.x.total_cmp(&b.0.x))
            .then(a.0.y.total_cmp(&b.0.y))
    });
    field.rebuild(&sources);
}
