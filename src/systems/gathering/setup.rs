use crate::entities::element_deposit::spawn_element_deposit;
use crate::resources::{ElementEconomy, GatheringRuntime, SimulationConfig, TrailIds};
use crate::types::Element;
use bevy::prelude::*;

pub fn spawn_deposits(commands: &mut Commands, config: &SimulationConfig) {
    let mut runtime = GatheringRuntime {
        random: config.seed,
        next_drop: 0,
    };
    let mut total = [0; 5];
    let mut positions: Vec<Vec2> = Vec::new();
    let towers: Vec<_> = (0..5)
        .map(|i| {
            Vec2::from_angle(i as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2)
                * 32.0_f32.min(config.half_extent * 0.75)
        })
        .collect();
    for index in 0..config.deposit_count {
        let extent = (config.half_extent - 2.0).max(0.0);
        let mut position = Vec2::ZERO;
        for _ in 0..512 {
            position = Vec2::new(runtime.unit() * 2.0 - 1.0, runtime.unit() * 2.0 - 1.0) * extent;
            if towers.iter().all(|p| p.distance_squared(position) >= 25.0)
                && positions
                    .iter()
                    .all(|p| p.distance_squared(position) >= 9.0)
            {
                break;
            }
        }
        positions.push(position);
        let amount = 5 + (runtime.unit() * 6.0) as u32;
        let element = Element::ALL[index % Element::ALL.len()];
        total[element as usize] += amount;
        spawn_element_deposit(commands, element, amount, position, index);
    }
    commands.insert_resource(runtime);
    commands.insert_resource(TrailIds::default());
    commands.insert_resource(ElementEconomy {
        initial: total,
        consumed: [0; 5],
    });
}
