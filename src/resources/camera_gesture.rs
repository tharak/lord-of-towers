use bevy::prelude::*;

/// World-space camera movement and pinch scale queued by touch input.
#[derive(Resource, Debug)]
pub struct CameraGesture {
    pub pan: Vec2,
    pub zoom: f32,
}
impl Default for CameraGesture {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}
