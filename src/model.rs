use bevy::prelude::*;
use std::collections::VecDeque;

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
/// The one tower whose faction accepts player equipment and production commands.
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
    ElementDeposit,
    Trail,
    DonutVisual,
    EquippedElementVisual,
    Projectile,
    StackVisual,
    TowerBeacon,
    TowerInfluenceVisual,
    EquipmentOrder,
    ProductionIndicator,
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
    pub minion_speed: f32,
    pub minion_health: f32,
    pub tower_health: f32,
    pub seed: u64,
    pub deposit_count: usize,
    pub vision_radius: f32,
    pub delivery_radius: f32,
    pub trail_spacing: f32,
    pub trail_lifetime: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub pickup_radius: f32,
}
impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            half_extent: 50.0,
            starting_minions: 3,
            tower_influence_radius: 15.0,
            minion_speed: 4.0,
            minion_health: 30.0,
            tower_health: 180.0,
            seed: 42,
            deposit_count: 30,
            vision_radius: 15.0,
            delivery_radius: 3.0,
            trail_spacing: 2.0,
            trail_lifetime: 30.0,
            projectile_speed: 20.0,
            projectile_lifetime: 1.5,
            pickup_radius: 1.0,
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
    PlayerCommands,
    EquipmentNavigation,
    Movement,
    Combat,
    Capture,
    Gathering,
    Production,
    EquipmentTransfers,
    Attributes,
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

/// A homing projectile. It follows a live hostile target and deals damage on impact.
#[derive(Component, Clone, Copy, Debug)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub remaining: f32,
}

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
pub const ELEMENTS_PER_MINION: usize = 5;

/// One canonical ordered inventory: deposit units, minion cargo (0..=1), or tower stock.
#[derive(Component, Clone, Debug, Default)]
pub struct ElementInventory {
    pub units: VecDeque<Element>,
}
impl ElementInventory {
    pub fn homogeneous(element: Element, amount: u32) -> Self {
        Self {
            units: std::iter::repeat_n(element, amount as usize).collect(),
        }
    }
    pub fn total(&self) -> usize {
        self.units.len()
    }
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
    pub fn counts(&self) -> [u32; 5] {
        let mut counts = [0; 5];
        for element in &self.units {
            counts[*element as usize] += 1;
        }
        counts
    }
}
#[derive(Component, Clone, Copy, Debug)]
pub struct ElementDeposit;

/// Elemental points belong to a minion, independently of its carried inventory.
#[derive(Component, Clone, Copy, Debug)]
pub struct ElementStats {
    pub fire: u32,
    pub water: u32,
    pub earth: u32,
    pub air: u32,
    pub void: u32,
}
impl Default for ElementStats {
    fn default() -> Self {
        Self {
            fire: 2,
            water: 2,
            earth: 2,
            air: 2,
            void: 2,
        }
    }
}
impl ElementStats {
    pub fn from_recipe(recipe: impl IntoIterator<Item = Element>) -> Self {
        let mut stats = Self::default();
        for element in recipe {
            match element {
                Element::Fire => stats.fire += 1,
                Element::Water => stats.water += 1,
                Element::Earth => stats.earth += 1,
                Element::Air => stats.air += 1,
                Element::Void => stats.void += 1,
            }
        }
        stats
    }
    pub fn equipped_multiplier(&self, element: Element, gear: &EquippedElements) -> f32 {
        let counts = gear.counts();
        let level = match element { Element::Fire => self.fire, Element::Water => self.water,
            Element::Earth => self.earth, Element::Air => self.air, Element::Void => return 1.0 };
        1.0 + 0.10 * level as f32 * counts[element as usize] as f32
            * (1.0 + 0.10 * self.void as f32 * counts[Element::Void as usize] as f32)
    }
}
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
/// Identity sequence for carrier trips.
#[derive(Resource, Default)]
pub struct TrailIds { pub next_trip: u64 }

#[derive(Resource, Debug)]
pub struct HudVisibility {
    pub visible: bool,
}
impl Default for HudVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
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
pub struct ElementEconomy {
    pub initial: [u32; 5],
    pub consumed: [u32; 5],
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

/// Three physical units, independent of gathering cargo and permanent birth stats.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EquippedElements { pub slots: [Option<Element>; 3] }
impl EquippedElements {
    pub fn counts(&self) -> [u32; 5] {
        let mut counts = [0; 5];
        for element in self.slots.iter().flatten() { counts[*element as usize] += 1; }
        counts
    }
}
#[derive(Clone, Copy, Debug)]
pub enum TowerOrderAction { Equip { slot: usize }, ReturnEquipment { slot: usize }, DeliverCargo }
/// Reservation lives in this entity's ElementInventory, never in spendable tower stock.
#[derive(Component, Clone, Copy, Debug)]
pub struct EquipmentOrder {
    pub minion: Entity,
    pub tower: Entity,
    pub source_position: Vec2,
    pub action: TowerOrderAction,
}
#[derive(Clone, Debug)]
pub enum PlayerCommand {
    Equip { minion: Entity, tower: Entity, slot: usize, element: Element },
    ReturnEquipment { minion: Entity, tower: Entity, slot: usize },
    DeliverCargo { minion: Entity, tower: Entity },
    CancelOrder { minion: Entity },
    CreateMinion { tower: Entity, counts: [u32; 5] },
}
#[derive(Resource, Default)]
pub struct PlayerCommands(pub Vec<PlayerCommand>);
#[derive(Resource, Default)]
pub struct PlayerSelection {
    pub entity: Option<Entity>,
    pub slot: usize,
    pub recipe: [u32; 5],
}
