//! Top-down sight footprints, independent of visual height or mesh geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) enum VisionBlockerShape {
    Rectangle,
    Ellipse,
}
