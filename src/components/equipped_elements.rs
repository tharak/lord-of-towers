//! ECS component data; behavior lives in systems.
use crate::types::{ELEMENT_BONUS_PERCENT, Element, VOID_BONUS_PERCENT};
use bevy::prelude::*;

/// Three physical units, independent of gathering cargo and permanent birth stats.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EquippedElements {
    pub slots: [Option<Element>; 3],
}
impl EquippedElements {
    /// Fixed equipment bonuses; birth-recipe points do not change these rates.
    pub fn multiplier(&self, element: Element) -> f32 {
        if element == Element::Void {
            return 1.0;
        }
        let counts = self.counts();
        1.0 + (ELEMENT_BONUS_PERCENT / 100.0)
            * counts[element as usize] as f32
            * (1.0 + (VOID_BONUS_PERCENT / 100.0) * counts[Element::Void as usize] as f32)
    }

    pub fn counts(&self) -> [u32; 5] {
        let mut counts = [0; 5];
        for element in self.slots.iter().flatten() {
            counts[*element as usize] += 1;
        }
        counts
    }
}
