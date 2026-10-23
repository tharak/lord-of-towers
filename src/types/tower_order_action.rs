#[derive(Clone, Copy, Debug)]
pub enum TowerOrderAction {
    Equip { slot: usize },
    ReturnEquipment { slot: usize },
    DeliverCargo,
}
