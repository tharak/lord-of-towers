use crate::resources::SimulationConfig;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
pub(crate) struct InputTiming<'w> {
    pub(crate) time: Res<'w, Time>,
    pub(crate) config: Res<'w, SimulationConfig>,
}
