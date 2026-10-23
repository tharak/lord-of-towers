use crate::{
    components::{Position, VisionBlocker},
    resources::VisionOcclusion,
};
use bevy::prelude::*;

/// Refresh only when map footprints change, before any minion perception this tick.
pub(crate) fn rebuild_vision_index(
    mut occlusion: ResMut<VisionOcclusion>,
    blockers: Query<(Entity, &Position, &VisionBlocker)>,
    changed: Query<
        (),
        (
            With<VisionBlocker>,
            Or<(Changed<Position>, Changed<VisionBlocker>)>,
        ),
    >,
    mut removed: RemovedComponents<VisionBlocker>,
) {
    let removed = removed.read().count() > 0;
    // Removal messages can expire during a pause or between widely spaced fixed ticks.
    if changed.is_empty() && !removed && blockers.iter().count() == occlusion.source_count() {
        return;
    }
    let mut entries: Vec<_> = blockers.iter().collect();
    entries.sort_unstable_by_key(|(entity, _, _)| entity.to_bits());
    occlusion.rebuild(
        entries
            .into_iter()
            .map(|(_, position, blocker)| (position.0, *blocker)),
    );
}
