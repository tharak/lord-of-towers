// ECS system parameters and query tuples naturally have wide signatures.
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod equipment;
pub mod influence;
pub mod inspector;
pub mod model;
pub mod orders;
pub mod presentation;
pub mod simulation;

pub mod gathering;

pub mod components;
pub mod entities;
pub mod plugins;
pub mod resources;
pub mod scheduling;
pub mod systems;
pub mod types;

mod integrations;
