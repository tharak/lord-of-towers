use crate::types::PlayerCommand;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerCommands(pub Vec<PlayerCommand>);
