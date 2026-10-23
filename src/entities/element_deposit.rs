//! Constructs element deposit entities; see the entity catalog for their component and lifecycle contract.
//! Match entities are replaced on restart; systems own behavior and destruction.
use crate::components::{
    ElementDeposit, ElementInventory, EntityKind, MatchEntity, Position, SimulationId,
};
use crate::resources::GatheringRuntime;
use crate::types::Element;
use bevy::prelude::*;

/// Death drops split mixed stock into homogeneous deposits without changing totals.
pub fn drop_cargo(commands: &mut Commands, position: Vec2, inventory: ElementInventory) {
    let counts = inventory.counts();
    if inventory.is_empty() {
        return;
    }
    commands.queue(move |world: &mut World| {
        for element in Element::ALL {
            let amount = counts[element as usize];
            if amount == 0 {
                continue;
            }
            let id = {
                let mut runtime = world.resource_mut::<GatheringRuntime>();
                let id = (1_u64 << 62) + runtime.next_drop;
                runtime.next_drop += 1;
                id
            };
            world.spawn((
                Name::new(format!("Dropped {}", element.name())),
                MatchEntity,
                EntityKind::ElementDeposit,
                ElementDeposit,
                Position(position),
                ElementInventory::homogeneous(element, amount),
                SimulationId(id),
            ));
        }
    });
}
pub(crate) fn spawn_element_deposit(
    commands: &mut Commands,
    element: Element,
    amount: u32,
    position: Vec2,
    index: usize,
) -> Entity {
    commands
        .spawn((
            Name::new(format!("{} deposit", element.name())),
            MatchEntity,
            EntityKind::ElementDeposit,
            ElementDeposit,
            Position(position),
            ElementInventory::homogeneous(element, amount),
            SimulationId((1 << 63) + index as u64),
        ))
        .id()
}
