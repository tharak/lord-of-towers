//! Energy systems run after combat/capture and before influence.
use crate::model::{EnergyClock, SimSet};
use bevy::prelude::*;

mod recharge;

pub struct EnergyPlugin;
impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyClock>().add_systems(
            FixedUpdate,
            recharge::recharge_energy.in_set(SimSet::Energy),
        );
    }
}
