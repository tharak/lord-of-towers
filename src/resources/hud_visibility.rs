use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct HudVisibility {
    pub visible: bool,
}
impl Default for HudVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
