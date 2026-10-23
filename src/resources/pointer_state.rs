use crate::types::PointerPress;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct PointerState {
    pub(crate) previous: Option<[(u64, Vec2); 2]>,
    pub(crate) suppress: bool,
    pub(crate) cooldown: f32,
    pub(crate) press: Option<PointerPress>,
}
