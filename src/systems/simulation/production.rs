use crate::components::{
    Allegiance, ElementInventory, ElementStats, Health, PlayerControlled, Position, SimulationId,
    Tower,
};
use crate::entities::minion::spawn_minion;
use crate::resources::{ElementEconomy, RandomStream, SimulationConfig, SpawnSequence};
use crate::types::ELEMENTS_PER_MINION;
use bevy::prelude::*;

/// Each birth consumes the oldest five units without changing existing minions.
pub(crate) fn produce_minions(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut random: ResMut<RandomStream>,
    mut sequence: ResMut<SpawnSequence>,
    mut economy: ResMut<ElementEconomy>,
    mut towers: Query<
        (
            Entity,
            &SimulationId,
            &Position,
            &Allegiance,
            &Health,
            &mut ElementInventory,
        ),
        (With<Tower>, Without<PlayerControlled>),
    >,
) {
    let mut ordered: Vec<_> = towers
        .iter()
        .map(|(entity, id, ..)| (id.0, entity))
        .collect();
    ordered.sort_by_key(|(id, _)| *id);
    for (_, entity) in ordered {
        let Ok((tower, _, position, owner, health, mut inventory)) = towers.get_mut(entity) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        while inventory.total() >= ELEMENTS_PER_MINION {
            let recipe: Vec<_> = inventory.units.drain(..ELEMENTS_PER_MINION).collect();
            for element in &recipe {
                economy.consumed[*element as usize] += 1;
            }
            spawn_minion(
                &mut commands,
                &config,
                &mut random,
                &mut sequence,
                tower,
                position.0,
                *owner,
                ElementStats::from_recipe(recipe),
            );
        }
    }
}
