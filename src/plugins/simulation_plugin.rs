//! Resource initialization and system ordering for this subsystem.
use crate::resources::{
    FactionEntities, MatchState, PendingDamage, PendingTowerAlerts, RandomStream, SimulationConfig,
    SpatialIndex, SpawnSequence,
};
use crate::scheduling::{SimSet, simulation_running};
use crate::systems::simulation::animation::update_arm_states;
use crate::systems::simulation::combat::{
    advance_projectiles, clear_pending_damage, collect_damage, resolve_damage,
};
use crate::systems::simulation::healing::heal_minions;
use crate::systems::simulation::match_lifecycle::{advance_clock, restart_match, setup_match};
use crate::systems::simulation::movement::{move_minions, prepare_drag};
use crate::systems::simulation::production::produce_minions;
use crate::systems::simulation::spatial::rebuild_index;
use crate::systems::simulation::targeting::{process_tower_alerts, select_combat_targets};
use crate::systems::simulation::victory::check_victory;
use bevy::prelude::*;

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationConfig>()
            .init_resource::<MatchState>()
            .init_resource::<FactionEntities>()
            .init_resource::<SpatialIndex>()
            .init_resource::<PendingDamage>()
            .init_resource::<PendingTowerAlerts>()
            .init_resource::<SpawnSequence>()
            .insert_resource(RandomStream(42))
            .insert_resource(Time::<Fixed>::from_hz(30.0))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::Spawn,
                    SimSet::PlayerCommands,
                    SimSet::Index,
                    SimSet::Target,
                    SimSet::EquipmentNavigation,
                    SimSet::CombatTarget,
                    SimSet::Movement,
                    SimSet::Combat,
                    SimSet::Capture,
                    SimSet::Gathering,
                    SimSet::EquipmentTransfers,
                    SimSet::Production,
                    SimSet::Attributes,
                    SimSet::Influence,
                    SimSet::Victory,
                )
                    .chain()
                    .run_if(simulation_running),
            )
            .add_systems(Startup, setup_match)
            .add_systems(PreUpdate, restart_match)
            .add_systems(FixedUpdate, advance_clock.in_set(SimSet::Spawn))
            .add_systems(FixedUpdate, rebuild_index.in_set(SimSet::Index))
            .add_systems(
                FixedUpdate,
                prepare_drag
                    .after(SimSet::Index)
                    .before(SimSet::Target)
                    .run_if(simulation_running),
            )
            .add_systems(
                FixedUpdate,
                heal_minions
                    .after(SimSet::Attributes)
                    .before(SimSet::Influence)
                    .run_if(simulation_running),
            )
            .add_systems(
                FixedUpdate,
                update_arm_states
                    .after(SimSet::Attributes)
                    .before(SimSet::Influence)
                    .run_if(simulation_running),
            )
            .add_systems(
                FixedUpdate,
                select_combat_targets.in_set(SimSet::CombatTarget),
            )
            .add_systems(FixedUpdate, move_minions.in_set(SimSet::Movement))
            .add_systems(
                FixedUpdate,
                (
                    rebuild_index,
                    clear_pending_damage,
                    advance_projectiles,
                    collect_damage,
                    process_tower_alerts,
                )
                    .chain()
                    .in_set(SimSet::Combat),
            )
            .add_systems(FixedUpdate, resolve_damage.in_set(SimSet::Capture))
            .add_systems(FixedUpdate, produce_minions.in_set(SimSet::Production))
            .add_systems(FixedUpdate, check_victory.in_set(SimSet::Victory));
    }
}
