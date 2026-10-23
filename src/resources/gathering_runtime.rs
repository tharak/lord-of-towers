use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct GatheringRuntime {
    pub(crate) random: u64,
    pub(crate) next_drop: u64,
}
impl Default for GatheringRuntime {
    fn default() -> Self {
        Self {
            random: 42,
            next_drop: 0,
        }
    }
}
impl GatheringRuntime {
    pub(crate) fn unit(&mut self) -> f32 {
        self.random = self.random.wrapping_add(0x9e3779b97f4a7c15);
        let mut n = self.random;
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((n ^ (n >> 31)) >> 40) as f32 / 16_777_216.0
    }
}
