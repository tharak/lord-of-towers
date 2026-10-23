use crate::components::VisionBlocker;
use bevy::prelude::*;
use std::collections::HashMap;

/// Bounded spatial cache of ECS sight footprints, containing no rendering state.
#[derive(Resource, Default)]
pub(crate) struct VisionOcclusion {
    blockers: Vec<(Vec2, VisionBlocker)>,
    cells: HashMap<IVec2, Vec<usize>>,
}
impl VisionOcclusion {
    const CELL_SIZE: f32 = 4.0;
    fn cell(position: Vec2) -> IVec2 {
        (position / Self::CELL_SIZE).floor().as_ivec2()
    }
    pub fn rebuild(&mut self, blockers: impl IntoIterator<Item = (Vec2, VisionBlocker)>) {
        self.blockers.clear();
        self.cells.clear();
        for (position, blocker) in blockers {
            if !position.is_finite()
                || !blocker.half_extents.is_finite()
                || !blocker.half_extents.cmpgt(Vec2::ZERO).all()
                || !blocker.angle.is_finite()
            {
                continue;
            }
            let bounds = blocker.bounds();
            let lo = Self::cell(position - bounds);
            let hi = Self::cell(position + bounds);
            let index = self.blockers.len();
            self.blockers.push((position, blocker));
            for y in lo.y..=hi.y {
                for x in lo.x..=hi.x {
                    self.cells.entry(IVec2::new(x, y)).or_default().push(index);
                }
            }
        }
    }
    pub fn has_line_of_sight(&self, from: Vec2, to: Vec2) -> bool {
        if !from.is_finite() || !to.is_finite() {
            return false;
        }
        if from.distance_squared(to) <= f32::EPSILON {
            return true;
        }
        let lo = Self::cell(from.min(to));
        let hi = Self::cell(from.max(to));
        // Only nearby grid buckets are inspected; exact footprint tests reject off-ray cells.
        for y in lo.y..=hi.y {
            for x in lo.x..=hi.x {
                if let Some(indices) = self.cells.get(&IVec2::new(x, y)) {
                    for &index in indices {
                        let (center, blocker) = self.blockers[index];
                        if blocker.blocks_segment(center, from, to) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}
