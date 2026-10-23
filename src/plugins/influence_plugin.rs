//! Resource initialization and system ordering for this subsystem.
use crate::resources::InfluenceField;
use crate::scheduling::SimSet;
use crate::systems::influence::territory::update_influence;
use bevy::prelude::*;

pub struct InfluencePlugin;
impl Plugin for InfluencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfluenceField>()
            .add_systems(FixedUpdate, update_influence.in_set(SimSet::Influence));
    }
}
