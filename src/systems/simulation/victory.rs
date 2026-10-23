use crate::components::{Allegiance, Faction, Tower};
use crate::resources::MatchState;
use bevy::prelude::*;

pub(crate) fn check_victory(
    towers: Query<&Allegiance, With<Tower>>,
    factions: Query<&Faction>,
    mut state: ResMut<MatchState>,
) {
    let mut ownership = towers.iter();
    let Some(first) = ownership.next() else {
        state.winner = None;
        state.paused = true;
        return;
    };
    if ownership.next().is_none()
        && let Ok(faction) = factions.get(first.0)
    {
        state.winner = Some(faction.index);
        state.paused = true;
    }
}
