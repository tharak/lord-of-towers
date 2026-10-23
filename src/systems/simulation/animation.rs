use crate::components::{ArmState, ElementInventory, Minion, ShootingPause};
use bevy::prelude::*;

pub(crate) fn update_arm_states(
    mut minions: Query<(&ElementInventory, &ShootingPause, &mut ArmState), With<Minion>>,
) {
    for (inventory, pause, mut arms) in &mut minions {
        arms.carrying = !inventory.is_empty();
        arms.shooting = pause.0 > 0.0;
    }
}
