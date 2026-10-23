//! Resource initialization and system ordering for this subsystem.
use crate::resources::{ElementEconomy, GatheringRuntime, TrailIds};
use crate::scheduling::SimSet;
use crate::systems::gathering::delivery::deliver;
use crate::systems::gathering::navigation::navigate;
use crate::systems::gathering::pickups::collect;
use crate::systems::gathering::trails::{decay, emit};
use bevy::prelude::*;

pub struct GatheringPlugin;
impl Plugin for GatheringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GatheringRuntime>()
            .init_resource::<ElementEconomy>()
            .init_resource::<TrailIds>()
            .add_systems(FixedUpdate, navigate.in_set(SimSet::Target))
            .add_systems(
                FixedUpdate,
                (decay, collect, deliver, emit)
                    .chain()
                    .in_set(SimSet::Gathering),
            );
    }
}
