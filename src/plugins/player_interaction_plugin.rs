//! Resource initialization and system ordering for this subsystem.
use crate::resources::{CameraGesture, PanelCache, PlayerCommands, PlayerSelection, PointerState};
use crate::systems::interaction::actions::context_actions;
use crate::systems::interaction::panel::context_panel;
use crate::systems::interaction::pointer::select_entity;
use bevy::prelude::*;

pub struct PlayerInteractionPlugin;
impl Plugin for PlayerInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSelection>()
            .init_resource::<PlayerCommands>()
            .init_resource::<CameraGesture>()
            .init_resource::<PointerState>()
            .init_resource::<PanelCache>()
            .add_systems(
                Update,
                (select_entity, context_actions, context_panel).chain(),
            );
    }
}
