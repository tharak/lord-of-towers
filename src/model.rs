use bevy::prelude::*;

pub const FACTION_NAMES: [&str; 5] = [
    "Orthanc",
    "Minas Morgul",
    "Minas Tirith",
    "Barad-dur",
    "Cirith Ungol",
];
pub const FACTION_RGB: [[u8; 3]; 5] = [
    [245, 144, 30],
    [30, 131, 245],
    [144, 30, 245],
    [11, 239, 132],
    [239, 245, 30],
];
pub fn faction_color(index: usize) -> Color {
    let [r, g, b] = FACTION_RGB[index];
    Color::srgb_u8(r, g, b)
}

#[derive(Component, Debug)]
pub struct Faction {
    pub index: usize,
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allegiance(pub Entity);
#[derive(Component, Clone, Copy, Debug)]
pub struct OriginTower(pub Entity);
#[derive(Component, Debug)]
pub struct Tower {
    pub slot: usize,
}
#[derive(Component, Debug)]
pub struct Minion;
#[derive(Component, Debug)]
pub struct MatchEntity;
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Faction,
    Tower,
    Minion,
    Battlefield,
    Camera,
    Light,
    Ui,
}
#[derive(Component, Clone, Copy, Debug)]
pub struct Position(pub Vec2);
#[derive(Component, Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}
#[derive(Component, Debug)]
pub struct Spawner {
    pub elapsed: f32,
    pub interval: f32,
    pub cap: usize,
}
#[derive(Component, Clone, Copy, Debug)]
pub struct Movement {
    pub speed: f32,
    pub target: Vec2,
    pub retarget_in: f32,
}
#[derive(Component, Clone, Copy, Debug)]
pub struct Attack {
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
    pub remaining: f32,
}
#[derive(Component, Clone, Copy, Debug)]
pub struct InfluenceEmitter {
    pub radius: f32,
    pub strength: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct SimulationConfig {
    pub half_extent: f32,
    pub spawn_interval: f32,
    pub population_cap: usize,
    pub minion_speed: f32,
    pub minion_health: f32,
    pub tower_health: f32,
    pub seed: u64,
}
impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            half_extent: 50.0,
            spawn_interval: 2.0,
            population_cap: 100,
            minion_speed: 4.0,
            minion_health: 30.0,
            tower_health: 180.0,
            seed: 42,
        }
    }
}
#[derive(Resource, Debug)]
pub struct MatchState {
    pub paused: bool,
    pub speed: f32,
    pub winner: Option<usize>,
    pub elapsed: f32,
    pub restart_requested: bool,
    pub browser_hidden: bool,
}
impl Default for MatchState {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1.0,
            winner: None,
            elapsed: 0.0,
            restart_requested: false,
            browser_hidden: false,
        }
    }
}
#[derive(Resource, Default)]
pub struct FactionEntities(pub Vec<Entity>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    Spawn,
    Index,
    Target,
    Movement,
    Combat,
    Capture,
    Influence,
    Victory,
}
pub fn simulation_running(state: Res<MatchState>) -> bool {
    !state.paused && !state.browser_hidden && state.winner.is_none()
}
pub fn simulation_delta(time: &Time<Fixed>, state: &MatchState) -> f32 {
    time.delta_secs() * state.speed
}
