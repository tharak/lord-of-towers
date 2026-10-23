use crate::systems::influence::territory::controller;
use crate::types::{INTERVAL, SIDE};
use bevy::prelude::*;

#[derive(Resource)]
pub struct InfluenceField {
    pub revision: u64,
    pub half_extent: f32,
    pub(crate) cells: Vec<[f32; 5]>,
    pub(crate) percentages: [f32; 5],
    pub(crate) elapsed: f32,
}
impl Default for InfluenceField {
    fn default() -> Self {
        Self {
            revision: 0,
            half_extent: 50.0,
            cells: vec![[0.0; 5]; SIDE * SIDE],
            percentages: [0.0; 5],
            elapsed: INTERVAL,
        }
    }
}
impl InfluenceField {
    /// Clear cached influence immediately when a match restarts, including while paused.
    pub fn clear(&mut self) {
        self.rebuild(&[]);
        self.elapsed = INTERVAL;
    }
    /// Bilinear sampling is continuous across cell boundaries. Outside the world is neutral.
    pub fn sample(&self, position: Vec2) -> [f32; 5] {
        if !position.is_finite() || position.abs().max_element() > self.half_extent {
            return [0.0; 5];
        }
        let uv = (position / self.half_extent + Vec2::ONE) * (0.5 * (SIDE - 1) as f32);
        let x = (uv.x.floor() as usize).min(SIDE - 2);
        let y = (uv.y.floor() as usize).min(SIDE - 2);
        let fraction = uv - Vec2::new(x as f32, y as f32);
        std::array::from_fn(|f| {
            let a = self.cells[y * SIDE + x][f];
            let b = self.cells[y * SIDE + x + 1][f];
            let c = self.cells[(y + 1) * SIDE + x][f];
            let d = self.cells[(y + 1) * SIDE + x + 1][f];
            (a + (b - a) * fraction.x) * (1.0 - fraction.y)
                + (c + (d - c) * fraction.x) * fraction.y
        })
    }
    /// Percent of the whole battlefield. Neutral and contested areas are not assigned.
    pub fn territory_percentages(&self) -> [f32; 5] {
        self.percentages
    }
    pub(crate) fn rebuild(&mut self, emitters: &[(Vec2, usize, f32, f32)]) {
        self.cells.fill([0.0; 5]);
        let spacing = self.half_extent * 2.0 / (SIDE - 1) as f32;
        // Touch only cells inside each emitter's support, avoiding units × whole-grid work.
        for &(position, faction, radius, strength) in emitters {
            if faction >= 5 || radius <= 0.0 || strength <= 0.0 {
                continue;
            }
            let lo = ((position - Vec2::splat(radius) + Vec2::splat(self.half_extent)) / spacing)
                .floor()
                .max(Vec2::ZERO);
            let hi = ((position + Vec2::splat(radius) + Vec2::splat(self.half_extent)) / spacing)
                .ceil()
                .min(Vec2::splat((SIDE - 1) as f32));
            for y in lo.y as usize..=hi.y.max(0.0) as usize {
                for x in lo.x as usize..=hi.x.max(0.0) as usize {
                    if x >= SIDE || y >= SIDE {
                        continue;
                    }
                    let point =
                        Vec2::new(x as f32, y as f32) * spacing - Vec2::splat(self.half_extent);
                    let falloff = (1.0 - point.distance(position) / radius).max(0.0);
                    self.cells[y * SIDE + x][faction] += strength * falloff * falloff;
                }
            }
        }
        let mut counts = [0usize; 5];
        for cell in &self.cells {
            if let Some(faction) = controller(*cell) {
                counts[faction] += 1;
            }
        }
        self.percentages = counts.map(|count| count as f32 * 100.0 / (SIDE * SIDE) as f32);
        self.revision = self.revision.wrapping_add(1);
    }
}
