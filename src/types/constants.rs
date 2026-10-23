use crate::components::EntityKind;

pub(crate) const TRAIL_BUCKETS: usize = 8;
pub(crate) const HEALTH_PERCENT_STEPS: usize = 100;
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
pub const ELEMENTS_PER_MINION: usize = 5;
pub const ELEMENT_BONUS_PERCENT: f32 = 30.0;
pub const VOID_BONUS_PERCENT: f32 = 10.0;
pub(crate) const KINDS: [EntityKind; 18] = [
    EntityKind::Faction,
    EntityKind::Tower,
    EntityKind::Minion,
    EntityKind::ElementDeposit,
    EntityKind::Trail,
    EntityKind::DonutVisual,
    EntityKind::EquippedElementVisual,
    EntityKind::Projectile,
    EntityKind::StackVisual,
    EntityKind::TowerBeacon,
    EntityKind::TowerInfluenceVisual,
    EntityKind::EquipmentOrder,
    EntityKind::ProductionIndicator,
    EntityKind::Battlefield,
    EntityKind::MapDecoration,
    EntityKind::Camera,
    EntityKind::Light,
    EntityKind::Ui,
];
pub(crate) const SIDE: usize = 101;
pub(crate) const INTERVAL: f32 = 0.1;
