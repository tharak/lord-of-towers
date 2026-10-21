//! Energy systems run after combat/capture and before influence.
use crate::model::{EnergyClock, SimSet};
use bevy::prelude::*;

mod network;
pub mod pickups;

pub struct EnergyPlugin;
impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyClock>().add_systems(
            FixedUpdate,
            (
                pickups::recharge_pickups,
                pickups::collect_pickups,
                network::reconcile_links,
                network::transfer_energy,
            )
                .chain()
                .in_set(SimSet::Energy),
        );
    }
}
