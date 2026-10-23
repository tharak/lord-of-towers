use crate::types::map_ui::MapTextRole;
use bevy::prelude::*;
#[derive(Component)]
pub(crate) struct MapSetupText(pub MapTextRole);
