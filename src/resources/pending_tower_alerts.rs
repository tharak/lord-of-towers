use crate::types::TowerAlert;
use bevy::prelude::*;

/// Firing alerts are drained after all attackers fire, before damage resolution.
#[derive(Resource, Default)]
pub struct PendingTowerAlerts(pub Vec<TowerAlert>);
