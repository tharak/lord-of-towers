//! ECS component data; behavior lives in systems.
use bevy::prelude::*;

/// Target shared by combat steering and firing, refreshed each target tick.
#[derive(Component, Clone, Copy, Debug)]
pub struct CombatTarget(pub Entity);
