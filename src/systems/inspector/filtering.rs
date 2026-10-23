use crate::components::{Allegiance, EntityKind, Faction};
use crate::types::KINDS;
use bevy::prelude::*;

pub(crate) fn matching_entities(
    world: &mut World,
    kind: usize,
    faction: usize,
    engine: bool,
) -> (usize, Vec<Entity>) {
    let mut query = world.query::<EntityRef>();
    let mut total = 0;
    let mut matched = Vec::new();
    for entity in query.iter(world) {
        total += 1;
        let entity_kind = entity.get::<EntityKind>();
        if entity_kind.is_none() && !engine {
            continue;
        }
        if kind != 0 && entity_kind != Some(&KINDS[kind - 1]) {
            continue;
        }
        if faction != 0 {
            let own_faction = entity.get::<Faction>().or_else(|| {
                entity
                    .get::<Allegiance>()
                    .and_then(|a| world.get::<Faction>(a.0))
            });
            if own_faction.is_none_or(|f| f.index != faction - 1) {
                continue;
            }
        }
        matched.push(entity.id());
    }
    matched.sort_unstable();
    (total, matched)
}
