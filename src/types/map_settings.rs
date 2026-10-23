//! Player-facing generation inputs. No generator-specific types cross this contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapPreset {
    Landscape,
    Town,
    Terrain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapRelief {
    Plains,
    Hills,
    Valley,
    Mountains,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapWater {
    None,
    Rivers,
    Lakes,
    RiversAndLakes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapDensity {
    None,
    Sparse,
    Dense,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapRoads {
    None,
    Regional,
    Streets,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapClimate {
    Temperate,
    Arid,
    Snow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapSettings {
    pub seed: u64,
    pub preset: MapPreset,
    pub relief: MapRelief,
    pub water: MapWater,
    pub vegetation: MapDensity,
    pub settlements: MapDensity,
    pub roads: MapRoads,
    pub climate: MapClimate,
}
impl Default for MapSettings {
    fn default() -> Self {
        Self::preset(MapPreset::Town, 42)
    }
}
impl MapSettings {
    fn preset(preset: MapPreset, seed: u64) -> Self {
        let mut settings = Self {
            seed,
            preset,
            relief: MapRelief::Hills,
            water: MapWater::RiversAndLakes,
            vegetation: MapDensity::Sparse,
            settlements: MapDensity::Sparse,
            roads: MapRoads::Streets,
            climate: MapClimate::Temperate,
        };
        match preset {
            MapPreset::Landscape => {}
            MapPreset::Town => {
                settings.relief = MapRelief::Plains;
                settings.water = MapWater::Rivers;
                settings.settlements = MapDensity::Dense;
            }
            MapPreset::Terrain => {
                settings.settlements = MapDensity::None;
                settings.roads = MapRoads::None;
            }
        }
        settings
    }
    pub fn apply_preset(&mut self, preset: MapPreset) {
        *self = Self::preset(preset, self.seed);
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct MapGenerationRequest {
    pub settings: MapSettings,
    pub half_extent: f32,
}
