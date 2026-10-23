//! ECS component data; behavior lives in systems.
use crate::types::Element;
use bevy::prelude::*;
use std::collections::VecDeque;

/// One canonical ordered inventory: deposit units, minion cargo (0..=1), or tower stock.
#[derive(Component, Clone, Debug, Default)]
pub struct ElementInventory {
    pub units: VecDeque<Element>,
}
impl ElementInventory {
    pub fn homogeneous(element: Element, amount: u32) -> Self {
        Self {
            units: std::iter::repeat_n(element, amount as usize).collect(),
        }
    }
    pub fn total(&self) -> usize {
        self.units.len()
    }
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
    pub fn counts(&self) -> [u32; 5] {
        let mut counts = [0; 5];
        for element in &self.units {
            counts[*element as usize] += 1;
        }
        counts
    }
}
