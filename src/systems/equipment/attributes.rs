use crate::components::{Attack, EquippedElements, Health, Minion, Movement, Vision};
use crate::resources::SimulationConfig;
use crate::types::Element;
use bevy::prelude::*;

pub(crate) fn recompute_attributes(
    config: Res<SimulationConfig>,
    mut minions: Query<
        (
            &EquippedElements,
            &mut Health,
            &mut Attack,
            &mut Movement,
            &mut Vision,
        ),
        With<Minion>,
    >,
) {
    for (gear, mut health, mut attack, mut movement, mut vision) in &mut minions {
        let fraction = if health.max > 0.0 {
            health.current / health.max
        } else {
            0.0
        };
        health.max = config.minion_health * gear.multiplier(Element::Earth);
        health.current = fraction * health.max;
        attack.damage = 10.0 * gear.multiplier(Element::Fire);
        movement.speed = config.minion_speed * gear.multiplier(Element::Water);
        vision.radius = config.vision_radius * gear.multiplier(Element::Air);
    }
}
