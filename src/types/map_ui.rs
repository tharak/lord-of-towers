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
