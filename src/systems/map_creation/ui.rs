use crate::components::{ControlBar, HudRoot, MapSetupAction, MapSetupRoot, MapSetupText};
use crate::resources::MapCreation;
use crate::types::{
    map_settings::{MapClimate, MapDensity, MapPreset, MapRelief, MapRoads, MapWater},
    map_ui::{MapPhase, MapTextRole},
};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
fn density(value: MapDensity) -> &'static str {
    match value {
        MapDensity::None => "None",
        MapDensity::Sparse => "Sparse",
        MapDensity::Dense => "Dense",
    }
}
pub(crate) fn update_map_ui(
    map: Res<MapCreation>,
    mut text: Query<(&MapSetupText, &mut Text)>,
    mut nodes: Query<(
        &mut Node,
        Option<&MapSetupRoot>,
        Option<&HudRoot>,
        Option<&ControlBar>,
    )>,
    mut buttons: Query<(&MapSetupAction, &mut BackgroundColor)>,
) {
    let playing = map.phase == MapPhase::Playing;
    let settings = &map.settings;
    for (role, mut text) in &mut text {
        text.0 = match role.0 {
            MapTextRole::Preset => format!(
                "Preset: {}",
                match settings.preset {
                    MapPreset::Landscape => "Landscape + villages",
                    MapPreset::Town => "Medieval town",
                    MapPreset::Terrain => "Terrain only",
                }
            ),
            MapTextRole::Relief => format!(
                "Relief: {}",
                match settings.relief {
                    MapRelief::Plains => "Plains",
                    MapRelief::Hills => "Hills",
                    MapRelief::Valley => "Valley",
                    MapRelief::Mountains => "Mountains",
                }
            ),
            MapTextRole::Water => format!(
                "Water: {}",
                match settings.water {
                    MapWater::None => "None",
                    MapWater::Rivers => "Rivers",
                    MapWater::Lakes => "Lakes",
                    MapWater::RiversAndLakes => "Rivers + lakes",
                }
            ),
            MapTextRole::Vegetation => format!("Vegetation: {}", density(settings.vegetation)),
            MapTextRole::Settlements => format!("Settlements: {}", density(settings.settlements)),
            MapTextRole::Roads => format!(
                "Roads: {}",
                match settings.roads {
                    MapRoads::None => "None",
                    MapRoads::Regional => "Regional",
                    MapRoads::Streets => "Regional + streets",
                }
            ),
            MapTextRole::Climate => format!(
                "Climate: {}",
                match settings.climate {
                    MapClimate::Temperate => "Temperate",
                    MapClimate::Arid => "Arid",
                    MapClimate::Snow => "Snow",
                }
            ),
            MapTextRole::Seed => format!("Seed: {}", settings.seed),
            MapTextRole::Status => map.message.clone(),
            MapTextRole::Launcher => if playing { "New map" } else { "Map settings" }.into(),
        };
    }
    for (mut node, panel, hud, bar) in &mut nodes {
        if panel.is_some() {
            node.display = if !playing && map.show_settings {
                Display::Flex
            } else {
                Display::None
            };
        }
        if !playing && (hud.is_some() || bar.is_some()) {
            node.display = Display::None;
        }
        if playing && bar.is_some() {
            node.display = Display::Flex;
        }
    }
    for (action, mut color) in &mut buttons {
        let disabled = match action {
            MapSetupAction::Start => map.phase != MapPhase::Ready,
            MapSetupAction::TogglePanel | MapSetupAction::NewMap => false,
            _ => map.phase == MapPhase::Generating,
        };
        color.0 = if disabled {
            Color::srgb(0.09, 0.12, 0.13)
        } else {
            Color::srgb(0.15, 0.26, 0.29)
        };
    }
}
pub(crate) fn scroll_map_settings(
    mut wheel: MessageReader<MouseWheel>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut previous: Local<Option<(u64, Vec2)>>,
    mut panels: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &Node,
            &mut ScrollPosition,
        ),
        With<MapSetupRoot>,
    >,
) {
    let delta: f32 = wheel
        .read()
        .map(|event| {
            event.y
                * if event.unit == MouseScrollUnit::Line {
                    24.0
                } else {
                    1.0
                }
        })
        .sum();
    let touch = if touches.iter().count() == 1 {
        touches.iter().next().map(|t| (t.id(), t.position()))
    } else {
        None
    };
    for (computed, transform, node, mut scroll) in &mut panels {
        if node.display == Display::None {
            continue;
        }
        let rect = Rect::from_center_size(
            transform.translation * computed.inverse_scale_factor(),
            computed.size() * computed.inverse_scale_factor(),
        );
        if windows
            .iter()
            .next()
            .and_then(|w| w.cursor_position())
            .is_some_and(|p| rect.contains(p))
        {
            scroll.0.y -= delta;
        }
        if let (Some((id, position)), Some((previous_id, previous_position))) = (touch, *previous) {
            if id == previous_id && rect.contains(position) {
                scroll.0.y += previous_position.y - position.y;
            }
        }
        let max = (computed.content_size().y - computed.size().y).max(0.0)
            * computed.inverse_scale_factor();
        scroll.0.y = scroll.0.y.clamp(0.0, max);
    }
    *previous = touch;
}
