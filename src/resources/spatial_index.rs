use crate::types::Neighbor;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub(crate) struct SpatialIndex(pub(crate) HashMap<(i32, i32), Vec<Neighbor>>);
impl SpatialIndex {
    const CELL: f32 = 8.0;
    pub(crate) fn cell(position: Vec2) -> (i32, i32) {
        (
            (position.x / Self::CELL).floor() as i32,
            (position.y / Self::CELL).floor() as i32,
        )
    }
    pub(crate) fn nearby(&self, position: Vec2, radius: f32) -> impl Iterator<Item = &Neighbor> {
        let (min_x, min_y) = Self::cell(position - Vec2::splat(radius));
        let (max_x, max_y) = Self::cell(position + Vec2::splat(radius));
        (min_x..=max_x)
            .flat_map(move |x| {
                (min_y..=max_y).flat_map(move |y| self.0.get(&(x, y)).into_iter().flatten())
            })
            .filter(move |entry| position.distance_squared(entry.position) <= radius * radius)
    }
}
