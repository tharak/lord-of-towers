use crate::components::{Allegiance, Faction, InfluenceEmitter, Position};
use crate::resources::{InfluenceField, MatchState, SimulationConfig};
use crate::scheduling::simulation_delta;
use crate::types::INTERVAL;
use bevy::prelude::*;

/// Near ties (within one percent of the strongest influence) remain contested.
pub fn controller(values: [f32; 5]) -> Option<usize> {
    let mut best = 0;
    for i in 1..5 {
        if values[i] > values[best] {
            best = i;
        }
    }
    let strongest = values[best];
    if strongest <= 0.00001 {
        return None;
    }
    if values
        .iter()
        .enumerate()
        .any(|(i, value)| i != best && strongest - value <= strongest * 0.01)
    {
        None
    } else {
        Some(best)
    }
}
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
