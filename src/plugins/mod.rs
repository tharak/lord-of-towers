//! Plugins source modules.

pub mod equipment_plugin;
pub mod gathering_plugin;
pub mod influence_plugin;
pub mod inspector_plugin;
pub mod player_interaction_plugin;
pub mod presentation_plugin;
pub mod simulation_plugin;

pub use equipment_plugin::EquipmentPlugin;
pub use gathering_plugin::GatheringPlugin;
pub use influence_plugin::InfluencePlugin;
pub use inspector_plugin::InspectorPlugin;
pub use player_interaction_plugin::PlayerInteractionPlugin;
pub use presentation_plugin::PresentationPlugin;
pub use simulation_plugin::SimulationPlugin;
