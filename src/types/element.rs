use bevy::prelude::*;

/// Resource type; ordering is shared by inventory counts and economy totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Element {
    Fire,
    Water,
    Earth,
    Air,
    Void,
}
impl Element {
    pub const ALL: [Self; 5] = [Self::Fire, Self::Water, Self::Earth, Self::Air, Self::Void];
    pub fn name(self) -> &'static str {
        match self {
            Self::Fire => "Fire",
            Self::Water => "Water",
            Self::Earth => "Earth",
            Self::Air => "Air",
            Self::Void => "Void",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Self::Fire => Color::srgb(0.784, 0.157, 0.118),
            Self::Water => Color::srgb(0.275, 0.314, 0.510),
            Self::Earth => Color::srgb(0.816, 0.627, 0.188),
            Self::Air => Color::srgb(0.275, 0.490, 0.549),
            Self::Void => Color::srgb(0.314, 0.314, 0.314),
        }
    }
}
