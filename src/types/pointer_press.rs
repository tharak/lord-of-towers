use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct PointerPress {
    pub(crate) touch: Option<u64>,
    pub(crate) start: Vec2,
    pub(crate) entity: Option<Entity>,
    pub(crate) minion: bool,
    pub(crate) dragging: bool,
    pub(crate) moved: bool,
}
