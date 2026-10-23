use crate::resources::MatchState;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    Spawn,
    Index,
    Target,
    CombatTarget,
    PlayerCommands,
    EquipmentNavigation,
    Movement,
    Combat,
    Capture,
    Gathering,
    Production,
    EquipmentTransfers,
    Attributes,
    Influence,
    Victory,
}
pub fn simulation_running(state: Res<MatchState>) -> bool {
    state.map_ready && !state.paused && !state.browser_hidden && state.winner.is_none()
}
pub fn simulation_delta(time: &Time<Fixed>, state: &MatchState) -> f32 {
    time.delta_secs() * state.speed
}
