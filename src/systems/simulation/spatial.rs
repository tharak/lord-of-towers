use crate::components::{Allegiance, Health, Minion, Position, SimulationId, Tower};
use crate::resources::SpatialIndex;
use crate::types::Neighbor;
use bevy::prelude::*;

pub(crate) fn rebuild_index(
    mut spatial: ResMut<SpatialIndex>,
    units: Query<
        (
            Entity,
            &Position,
            &Allegiance,
            Option<&Tower>,
            Option<&SimulationId>,
            Option<&Health>,
        ),
        Or<(With<Tower>, With<Minion>)>,
    >,
) {
    spatial.0.clear();
    for (entity, position, faction, tower, stable_id, health) in &units {
        if health.is_some_and(|health| health.current <= 0.0) {
            continue;
        }
        spatial
            .0
            .entry(SpatialIndex::cell(position.0))
            .or_default()
            .push(Neighbor {
                entity,
                position: position.0,
                faction: faction.0,
                tower: tower.is_some(),
                stable_id: stable_id.map_or(entity.to_bits(), |id| id.0),
            });
    }
    for bucket in spatial.0.values_mut() {
        bucket.sort_by_key(|entry| entry.stable_id);
    }
}
