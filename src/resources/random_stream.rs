use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct RandomStream(pub(crate) u64);
impl RandomStream {
    pub(crate) fn unit(&mut self) -> f32 {
        // SplitMix64 also handles seed zero. This sequence is platform-independent.
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut n = self.0;
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((n ^ (n >> 31)) >> 40) as f32 / 16_777_216.0
    }
    pub(crate) fn direction(&mut self) -> Vec2 {
        Vec2::from_angle(self.unit() * std::f32::consts::TAU)
    }
}
