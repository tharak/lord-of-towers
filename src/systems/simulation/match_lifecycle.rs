use crate::components::{Allegiance, ElementStats, MatchEntity};
use crate::entities::faction::spawn_faction;
use crate::entities::minion::spawn_minion;
use crate::entities::tower::spawn_tower;
use crate::resources::{
    FactionEntities, InfluenceField, MatchState, PendingDamage, PendingTowerAlerts, PlayerCommands,
    PlayerSelection, RandomStream, SimulationConfig, SpatialIndex, SpawnSequence, TrailIds,
};
use crate::scheduling::simulation_delta;
use crate::systems::gathering::setup::spawn_deposits;
use crate::types::FACTION_NAMES;
use bevy::prelude::*;

pub(crate) fn setup_match(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut factions: ResMut<FactionEntities>,
    mut random: ResMut<RandomStream>,
    mut sequence: ResMut<SpawnSequence>,
) {
    sequence.0 = 5;
    create_match(
        &mut commands,
        &config,
        &mut factions,
        &mut random,
        &mut sequence,
    );
}
pub(crate) fn create_match(
    commands: &mut Commands,
    config: &SimulationConfig,
    factions: &mut FactionEntities,
    random: &mut RandomStream,
    sequence: &mut SpawnSequence,
) {
    random.0 = config.seed;
    factions.0.clear();
    for (index, name) in FACTION_NAMES.iter().enumerate() {
        let faction = spawn_faction(commands, index, name);
        factions.0.push(faction);
        let angle = index as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
        let position = Vec2::from_angle(angle) * 32.0_f32.min(config.half_extent * 0.75);
        let tower = spawn_tower(commands, config, index, name, faction, position);
        for _ in 0..config.starting_minions {
            spawn_minion(
                commands,
                config,
                random,
                sequence,
                tower,
                position,
                Allegiance(faction),
                ElementStats::default(),
            );
        }
    }
    spawn_deposits(commands, config);
}
pub(crate) fn restart_match(
    mut commands: Commands,
    mut state: ResMut<MatchState>,
    config: Res<SimulationConfig>,
    entities: Query<Entity, With<MatchEntity>>,
    mut factions: ResMut<FactionEntities>,
    mut random: ResMut<RandomStream>,
    mut spatial: ResMut<SpatialIndex>,
    mut damage: ResMut<PendingDamage>,
    mut alerts: ResMut<PendingTowerAlerts>,
    field: Option<ResMut<InfluenceField>>,
    mut sequence: ResMut<SpawnSequence>,
    mut player_commands: ResMut<PlayerCommands>,
    mut selection: ResMut<PlayerSelection>,
    mut trails: ResMut<TrailIds>,
) {
    if !state.restart_requested {
        return;
    }
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    let hidden = state.browser_hidden;
    *state = MatchState {
        browser_hidden: hidden,
        ..default()
    };
    spatial.0.clear();
    damage.0.clear();
    alerts.0.clear();
    sequence.0 = 5;
    player_commands.0.clear();
    *selection = PlayerSelection::default();
    *trails = TrailIds::default();
    if let Some(mut field) = field {
        field.clear();
    }
    create_match(
        &mut commands,
        &config,
        &mut factions,
        &mut random,
        &mut sequence,
    );
}
pub(crate) fn advance_clock(time: Res<Time<Fixed>>, mut state: ResMut<MatchState>) {
    state.elapsed += simulation_delta(&time, &state);
}
