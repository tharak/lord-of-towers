//! Resource initialization and system ordering for this subsystem.
use crate::entities::ui::inspector::spawn_inspector_ui;
use crate::resources::{InspectorFilter, InspectorState};
use crate::systems::inspector::controls::controls;
use crate::systems::inspector::details::refresh;
use bevy::prelude::*;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorState>()
            .init_resource::<InspectorFilter>()
            .add_systems(Startup, spawn_inspector_ui)
            .add_systems(Update, (controls, refresh).chain());
    }
}
