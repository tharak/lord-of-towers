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
        Self {
            seed: 42,
            preset: MapPreset::Landscape,
            relief: MapRelief::Hills,
            water: MapWater::RiversAndLakes,
            vegetation: MapDensity::Sparse,
            settlements: MapDensity::Sparse,
            roads: MapRoads::Streets,
            climate: MapClimate::Temperate,
        }
    }
}
impl MapSettings {
    pub fn apply_preset(&mut self, preset: MapPreset) {
        let seed = self.seed;
        *self = Self::default();
        self.seed = seed;
        self.preset = preset;
        match preset {
            MapPreset::Landscape => {}
            MapPreset::Town => {
                self.relief = MapRelief::Plains;
                self.water = MapWater::Rivers;
                self.vegetation = MapDensity::Sparse;
                self.settlements = MapDensity::Dense;
            }
            MapPreset::Terrain => {
                self.settlements = MapDensity::None;
                self.roads = MapRoads::None;
            }
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct MapGenerationRequest {
    pub settings: MapSettings,
    pub half_extent: f32,
}
