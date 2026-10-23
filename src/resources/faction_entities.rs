use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct FactionEntities(pub Vec<Entity>);
