use bevy::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct SimulationConfig {
    pub half_extent: f32,
    pub starting_minions: usize,
    pub tower_influence_radius: f32,
    pub minion_speed: f32,
    pub minion_health: f32,
    pub tower_health: f32,
    pub seed: u64,
    pub deposit_count: usize,
    pub vision_radius: f32,
    pub delivery_radius: f32,
    pub trail_spacing: f32,
    pub trail_lifetime: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub pickup_radius: f32,
}
impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            half_extent: 50.0,
            starting_minions: 3,
            tower_influence_radius: 15.0,
            minion_speed: 4.0,
            minion_health: 90.0,
            tower_health: 180.0,
            seed: 42,
            deposit_count: 30,
            vision_radius: 15.0,
            delivery_radius: 3.0,
            trail_spacing: 2.0,
            trail_lifetime: 30.0,
            projectile_speed: 20.0,
            projectile_lifetime: 1.5,
            pickup_radius: 1.0,
        }
    }
}
