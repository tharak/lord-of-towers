//! Renderer-independent output of the map-generation boundary.
use bevy::prelude::*;
#[derive(Clone, Copy, Debug)]
pub(crate) enum MapPropKind {
    Building,
    Tree,
    Rock,
}
#[derive(Clone, Debug)]
pub(crate) struct MapProp {
    pub kind: MapPropKind,
    pub position: Vec2,
    pub size: Vec3,
    pub angle: f32,
}
#[derive(Clone, Debug)]
pub(crate) struct MapData {
    pub resolution: usize,
    pub half_extent: f32,
    /// Linear RGB, row-major from the negative-Z edge of the battlefield.
    pub colors: Vec<[f32; 3]>,
    pub props: Vec<MapProp>,
    pub summary: String,
}
impl MapData {
    pub fn color_at(&self, position: Vec2) -> [f32; 3] {
        let uv = ((position / self.half_extent + Vec2::ONE) * 0.5).clamp(Vec2::ZERO, Vec2::ONE)
            * (self.resolution - 1) as f32;
        let x = (uv.x.floor() as usize).min(self.resolution - 2);
        let y = (uv.y.floor() as usize).min(self.resolution - 2);
        let tx = uv.x - x as f32;
        let ty = uv.y - y as f32;
        std::array::from_fn(|channel| {
            let a = self.colors[y * self.resolution + x][channel];
            let b = self.colors[y * self.resolution + x + 1][channel];
            let c = self.colors[(y + 1) * self.resolution + x][channel];
            let d = self.colors[(y + 1) * self.resolution + x + 1][channel];
            (a + (b - a) * tx) * (1.0 - ty) + (c + (d - c) * tx) * ty
        })
    }
}
