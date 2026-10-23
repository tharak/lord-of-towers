//! Resource initialization and system ordering for this subsystem.
use crate::resources::{PlayerCommands, PlayerSelection};
use crate::scheduling::SimSet;
use crate::systems::equipment::attributes::recompute_attributes;
use crate::systems::equipment::commands::process_commands;
use crate::systems::equipment::navigation::navigate_orders;
use crate::systems::equipment::npc::equip_npcs;
use crate::systems::equipment::reservations::cleanup_orders;
use crate::systems::equipment::transfers::transfer_orders;
use crate::systems::simulation::match_lifecycle::restart_match;
use bevy::prelude::*;

pub struct EquipmentPlugin;
impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommands>()
            .init_resource::<PlayerSelection>()
            .add_systems(PreUpdate, cleanup_orders.after(restart_match))
            .add_systems(
                FixedUpdate,
                (process_commands, recompute_attributes)
                    .chain()
                    .in_set(SimSet::PlayerCommands),
            )
            .add_systems(
                FixedUpdate,
                navigate_orders.in_set(SimSet::EquipmentNavigation),
            )
            .add_systems(
                FixedUpdate,
                (transfer_orders, equip_npcs)
                    .chain()
                    .in_set(SimSet::EquipmentTransfers),
            )
            .add_systems(FixedUpdate, recompute_attributes.in_set(SimSet::Attributes))
            .add_systems(
                FixedUpdate,
                cleanup_orders
                    .before(SimSet::Victory)
                    .after(SimSet::Attributes),
            );
    }
}
