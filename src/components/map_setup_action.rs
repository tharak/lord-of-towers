use bevy::prelude::*;
#[derive(Component, Clone, Copy)]
pub(crate) enum MapSetupAction {
    Preset,
    Relief,
    Water,
    Vegetation,
    Settlements,
    Roads,
    Climate,
    SeedPrevious,
    SeedNext,
    SeedShuffle,
    Generate,
    Start,
    TogglePanel,
    NewMap,
}
