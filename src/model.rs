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
/// The one tower whose faction accepts player-drawn trails.
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerControlled;
#[derive(Component, Debug)]
pub struct Minion;
#[derive(Component, Debug)]
pub struct MatchEntity;
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Faction,
    Tower,
    Minion,
    RawElementDeposit,
    Trail,
    DonutVisual,
    EnergyOrbVisual,
    EnergyProjectile,
    StackVisual,
    TowerBeacon,
    TowerInfluenceVisual,
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
/// Movement pause after firing; counted in simulation seconds.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ShootingPause(pub f32);
#[derive(Component, Clone, Copy, Debug)]
pub struct InfluenceEmitter {
    pub radius: f32,
    pub strength: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct SimulationConfig {
    pub half_extent: f32,
    pub starting_minions: usize,
    pub tower_influence_radius: f32,
    pub tower_radius_per_raw: f32,
    pub drawn_trail_capacity: usize,
    pub minion_speed: f32,
    pub minion_health: f32,
    pub tower_health: f32,
    pub seed: u64,
    pub deposit_count: usize,
    pub vision_radius: f32,
    pub delivery_radius: f32,
    pub trail_spacing: f32,
    pub trail_lifetime: f32,
    pub initial_energy: u8,
    pub energy_capacity: u8,
    pub attack_energy_cost: u8,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub pickup_radius: f32,
    pub energy_tick_seconds: f32,
    pub energy_recharge_ticks: u32,
}
impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            half_extent: 50.0,
            starting_minions: 3,
            tower_influence_radius: 15.0,
            tower_radius_per_raw: 1.0,
            drawn_trail_capacity: 10,
            minion_speed: 4.0,
            minion_health: 30.0,
            tower_health: 180.0,
            seed: 42,
            deposit_count: 30,
            vision_radius: 15.0,
            delivery_radius: 3.0,
            trail_spacing: 2.0,
            trail_lifetime: 30.0,
            initial_energy: 5,
            energy_capacity: 5,
            attack_energy_cost: 1,
            projectile_speed: 20.0,
            projectile_lifetime: 1.5,
            pickup_radius: 1.0,
            energy_tick_seconds: 1.0,
            energy_recharge_ticks: 5,
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
    CombatTarget,
    TrailInput,
    Movement,
    Combat,
    Capture,
    Gathering,
    TowerGrowth,
    Energy,
    Influence,
    Victory,
}
pub fn simulation_running(state: Res<MatchState>) -> bool {
    !state.paused && !state.browser_hidden && state.winner.is_none()
}
pub fn simulation_delta(time: &Time<Fixed>, state: &MatchState) -> f32 {
    time.delta_secs() * state.speed
}

/// Stable across match restart; also resolves deterministic combat and collection ties.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimulationId(pub u64);

#[derive(Component, Clone, Copy, Debug)]
pub struct EnergyStore {
    pub current: u8,
    pub capacity: u8,
}

/// A fired unit of energy. It follows a live hostile target and deals damage on impact.
#[derive(Component, Clone, Copy, Debug)]
pub struct EnergyProjectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub remaining: f32,
}

/// Gameplay inventory: deposits, minion cargo (0..=1), or tower stock.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct RawElement(pub u32);
#[derive(Component, Clone, Copy, Debug)]
pub struct RawElementDeposit;
#[derive(Component, Clone, Copy, Debug)]
pub struct Vision {
    pub radius: f32,
}
#[derive(Component, Clone, Copy, Debug, Default)]
pub enum GatheringState {
    #[default]
    Exploring,
    Following {
        trip: u64,
        sequence: u32,
    },
    Returning {
        tower: Option<Entity>,
    },
}
#[derive(Component, Clone, Copy, Debug)]
pub struct Trail {
    pub trip: u64,
    pub sequence: u32,
    pub remaining: f32,
}
/// Origin metadata for the player marker budget and whole-stroke erasing.
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerDrawnTrail {
    pub serial: u64,
}

/// Completed pointer gestures, each an ordered world-space polyline, queued for simulation.
#[derive(Resource, Default)]
pub struct DrawTrailRequests(pub Vec<Vec<Vec2>>);

/// Shared identity sequence for carrier trips and player strokes.
#[derive(Resource, Default)]
pub struct TrailIds {
    pub next_trip: u64,
    pub next_player_marker: u64,
}

/// Locations pending erasure of the closest player-drawn stroke.
#[derive(Resource, Default)]
pub struct EraseTrailRequests(pub Vec<Vec2>);

#[derive(Resource, Debug)]
pub struct HudVisibility {
    pub visible: bool,
}
impl Default for HudVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

/// The battlefield tool selected by the player's touch or mouse controls.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailTool {
    #[default]
    Draw,
    Erase,
}

/// World-space camera movement and pinch scale queued by touch input.
#[derive(Resource, Debug)]
pub struct CameraGesture {
    pub pan: Vec2,
    pub zoom: f32,
}
impl Default for CameraGesture {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}
#[derive(Component, Clone, Copy, Debug)]
pub struct TrailEmitter {
    pub trip: u64,
    pub next_sequence: u32,
    pub last_position: Vec2,
}
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct RawElementEconomy {
    pub initial: u32,
}

#[derive(Resource, Debug, Default)]
pub struct EnergyClock {
    pub elapsed: f32,
    pub pulses: u64,
}

/// Target shared by combat steering and firing, refreshed each target tick.
#[derive(Component, Clone, Copy, Debug)]
pub struct CombatTarget(pub Entity);

/// Persistent response to a visible attacker that fired at a friendly tower.
#[derive(Component, Clone, Copy, Debug)]
pub struct TowerDefenseTarget {
    pub tower: Entity,
    pub attacker: Entity,
}

#[derive(Clone, Copy, Debug)]
pub struct TowerAlert {
    pub tower: Entity,
    pub attacker: Entity,
}

/// Firing alerts are drained after all attackers fire, before damage resolution.
#[derive(Resource, Default)]
pub struct PendingTowerAlerts(pub Vec<TowerAlert>);
