use crate::components::InspectorAction;
use crate::resources::{InspectorFilter, InspectorState};
use crate::types::KINDS;
use bevy::prelude::*;

pub(crate) fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    actions: Query<(&Interaction, &InspectorAction), Changed<Interaction>>,
    mut state: ResMut<InspectorState>,
    mut filter: ResMut<InspectorFilter>,
) {
    if keys.just_pressed(KeyCode::F3) {
        state.open = !state.open;
        filter.next_refresh = 0.0;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            InspectorAction::Kind => {
                filter.kind = (filter.kind + 1) % (KINDS.len() + 1);
                filter.page = 0;
            }
            InspectorAction::Faction => {
                filter.faction = (filter.faction + 1) % 6;
                filter.page = 0;
            }
            InspectorAction::Engine => {
                filter.engine = !filter.engine;
                filter.page = 0;
            }
            InspectorAction::Previous => {
                filter.page = filter.page.saturating_sub(1);
                filter.component_page = 0;
            }
            InspectorAction::Next => {
                filter.page += 1;
                filter.component_page = 0;
            }
            InspectorAction::Components => filter.component_page += 1,
            InspectorAction::Close => state.open = false,
        }
        filter.next_refresh = 0.0;
    }
}
