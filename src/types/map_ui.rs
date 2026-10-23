#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MapPhase {
    Configuring,
    Generating,
    Ready,
    Playing,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum MapTextRole {
    Preset,
    Relief,
    Water,
    Vegetation,
    Settlements,
    Roads,
    Climate,
    Seed,
    Status,
    Launcher,
}

/// A touch is a button tap only if it never crosses the scroll threshold.
pub(crate) struct MapTouchPress {
    pub entity: bevy::prelude::Entity,
    pub touch: u64,
    pub start: bevy::prelude::Vec2,
    pub moved: bool,
}
