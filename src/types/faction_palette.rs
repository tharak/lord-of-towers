use crate::types::FACTION_RGB;
use bevy::prelude::*;

pub fn faction_color(index: usize) -> Color {
    let [r, g, b] = FACTION_RGB[index];
    Color::srgb_u8(r, g, b)
}
