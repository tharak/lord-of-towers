use bevy::prelude::*;

/// Identity sequence for carrier trips.
#[derive(Resource, Default)]
pub struct TrailIds {
    pub next_trip: u64,
}
