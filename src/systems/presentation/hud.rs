use crate::components::{
    Allegiance, ElementInventory, Faction, FactionText, Minion, StatusText, Tower,
};
use crate::resources::{InfluenceField, MatchState};
use crate::systems::gathering::navigation::units;
use crate::types::FACTION_NAMES;
use bevy::prelude::*;

pub(crate) fn update_hud(
    state: Res<MatchState>,
    field: Res<InfluenceField>,
    factions: Query<&Faction>,
    units: Query<
        (&Allegiance, Option<&Tower>, Option<&ElementInventory>),
        Or<(With<Tower>, With<Minion>)>,
    >,
    mut texts: Query<(&mut Text, Option<&FactionText>), Or<(With<StatusText>, With<FactionText>)>>,
) {
    let mut towers = [0; 5];
    let mut minions = [0; 5];
    let mut stock = [0; 5];
    for (allegiance, tower, raw) in &units {
        if let Ok(faction) = factions.get(allegiance.0) {
            if tower.is_some() {
                towers[faction.index] += 1;
                stock[faction.index] += raw.map_or(0, ElementInventory::total);
            } else {
                minions[faction.index] += 1;
            }
        }
    }
    let territory = field.territory_percentages();
    for (mut text, faction) in &mut texts {
        text.0 = if let Some(FactionText(i)) = faction {
            format!(
                "{}  |  {} towers | {} minions | Element total {} | {:.0}%",
                FACTION_NAMES[*i], towers[*i], minions[*i], stock[*i], territory[*i]
            )
        } else if let Some(winner) = state.winner {
            format!("{} wins! Restart to play again.", FACTION_NAMES[winner])
        } else {
            format!(
                "{} | {:.0}x speed | {:.0}s",
                if state.paused { "Paused" } else { "Expanding" },
                state.speed,
                state.elapsed
            )
        };
    }
}
