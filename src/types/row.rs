use crate::components::ContextAction;
use bevy::prelude::*;

pub(crate) struct Row {
    pub(crate) label: String,
    pub(crate) action: Option<ContextAction>,
    pub(crate) color: Color,
}
