use crate::types::VisionBlockerShape;
use bevy::prelude::*;

/// Static opaque footprint. Together with Position, defines sight obstruction only.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct VisionBlocker {
    pub half_extents: Vec2,
    /// Same world-Y rotation as the corresponding map prop.
    pub angle: f32,
    pub shape: VisionBlockerShape,
}
impl VisionBlocker {
    pub fn bounds(self) -> Vec2 {
        let (sin, cos) = self.angle.sin_cos();
        Vec2::new(
            cos.abs() * self.half_extents.x + sin.abs() * self.half_extents.y,
            sin.abs() * self.half_extents.x + cos.abs() * self.half_extents.y,
        )
    }

    pub fn blocks_segment(self, center: Vec2, from: Vec2, to: Vec2) -> bool {
        // X/Z coordinates rotate oppositely to the usual 2D X/Y convention.
        let inverse = Mat2::from_angle(self.angle);
        let from = inverse * (from - center);
        let to = inverse * (to - center);
        let inside = |point: Vec2| match self.shape {
            VisionBlockerShape::Rectangle => point.abs().cmple(self.half_extents).all(),
            VisionBlockerShape::Ellipse => (point / self.half_extents).length_squared() <= 1.0,
        };
        // Sharing the same building interior/canopy does not hide occupants from each other.
        if inside(from) && inside(to) {
            return false;
        }
        match self.shape {
            VisionBlockerShape::Rectangle => {
                let delta = to - from;
                let mut entry: f32 = 0.0;
                let mut exit: f32 = 1.0;
                for axis in 0..2 {
                    if delta[axis].abs() <= f32::EPSILON {
                        if from[axis].abs() > self.half_extents[axis] {
                            return false;
                        }
                    } else {
                        let a = (-self.half_extents[axis] - from[axis]) / delta[axis];
                        let b = (self.half_extents[axis] - from[axis]) / delta[axis];
                        entry = entry.max(a.min(b));
                        exit = exit.min(a.max(b));
                        if entry > exit {
                            return false;
                        }
                    }
                }
                true
            }
            VisionBlockerShape::Ellipse => {
                let from = from / self.half_extents;
                let to = to / self.half_extents;
                let delta = to - from;
                let length = delta.length_squared();
                let t = if length > f32::EPSILON {
                    (-from.dot(delta) / length).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (from + delta * t).length_squared() <= 1.0
            }
        }
    }
}
