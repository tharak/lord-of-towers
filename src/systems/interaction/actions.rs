use crate::components::{ContextAction, ElementInventory, Health, Tower};
use crate::resources::{MatchState, PlayerCommands, PlayerSelection};
use crate::systems::interaction::panel::recipe_stock;
use crate::types::{ELEMENTS_PER_MINION, PlayerCommand};
use bevy::prelude::*;

pub(crate) fn context_actions(
    actions: Query<(&Interaction, &ContextAction), Changed<Interaction>>,
    mut selection: ResMut<PlayerSelection>,
    mut commands: ResMut<PlayerCommands>,
    state: Res<MatchState>,
    towers: Query<(&ElementInventory, &Health), With<Tower>>,
) {
    if state.restart_requested || state.browser_hidden || state.winner.is_some() {
        return;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            ContextAction::Equipment { command, cancel } => {
                if *cancel {
                    if let PlayerCommand::Equip { minion, .. }
                    | PlayerCommand::ReturnEquipment { minion, .. } = command
                    {
                        commands
                            .0
                            .push(PlayerCommand::CancelOrder { minion: *minion });
                    }
                }
                commands.0.push(command.clone());
            }
            ContextAction::Recipe { tower, element } => {
                if selection.entity != Some(*tower) {
                    continue;
                }
                let Ok((inventory, health)) = towers.get(*tower) else {
                    continue;
                };
                if health.current <= 0.0 {
                    continue;
                }
                let stock = recipe_stock(inventory, *tower, &commands);
                for (selected, available) in selection.recipe.iter_mut().zip(stock) {
                    *selected = (*selected).min(available);
                }
                let index = *element as usize;
                if selection.recipe[index] >= stock[index] {
                    continue;
                }
                selection.recipe[index] += 1;
                if selection.recipe.iter().sum::<u32>() == ELEMENTS_PER_MINION as u32 {
                    commands.0.push(PlayerCommand::CreateMinion {
                        tower: *tower,
                        counts: selection.recipe,
                    });
                    selection.recipe = [0; 5];
                }
            }
        }
    }
}
