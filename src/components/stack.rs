//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Reusable primitive stack: count, grid layout, and spacing are presentation only.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Stack {
    pub(crate) count: usize,
    pub(crate) columns: usize,
    pub(crate) spacing: Vec3,
}
impl Stack {
    pub(crate) fn transform(self, index: usize) -> Transform {
        let width = self.columns;
        let layer = width * width;
        Transform::from_xyz(
            (index % width) as f32 * self.spacing.x - (width - 1) as f32 * self.spacing.x / 2.0,
            (index / layer) as f32 * self.spacing.y + self.spacing.y / 2.0,
            ((index / width) % width) as f32 * self.spacing.z
                - (width - 1) as f32 * self.spacing.z / 2.0,
        )
    }
}
