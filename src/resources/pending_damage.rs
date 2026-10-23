use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub(crate) struct PendingDamage(pub(crate) HashMap<Entity, [f32; 5]>);
