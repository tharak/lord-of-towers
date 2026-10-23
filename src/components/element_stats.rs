//! ECS component data; behavior lives in systems.
use crate::types::Element;
use bevy::prelude::*;

/// Elemental points belong to a minion, independently of its carried inventory.
#[derive(Component, Clone, Copy, Debug)]
pub struct ElementStats {
    pub fire: u32,
    pub water: u32,
    pub earth: u32,
    pub air: u32,
    pub void: u32,
}
impl Default for ElementStats {
    fn default() -> Self {
        Self {
            fire: 2,
            water: 2,
            earth: 2,
            air: 2,
            void: 2,
        }
    }
}
impl ElementStats {
    pub fn from_recipe(recipe: impl IntoIterator<Item = Element>) -> Self {
        let mut stats = Self::default();
        for element in recipe {
            match element {
                Element::Fire => stats.fire += 1,
                Element::Water => stats.water += 1,
                Element::Earth => stats.earth += 1,
                Element::Air => stats.air += 1,
                Element::Void => stats.void += 1,
            }
        }
        stats
    }
}
