#[cfg(target_arch = "wasm32")]
use crate::components::{
    Control, DonutVisual, ElementDeposit, ElementInventory, EntityKind, EquippedElementVisual,
    InfluenceEmitter, Minion, PlayerControlled, Projectile, SpectatorCamera, StackPiece, Tower,
    TowerBeacon, TowerInfluenceVisual, Trail,
};
#[cfg(target_arch = "wasm32")]
use crate::resources::{ElementEconomy, HudVisibility, InspectorState, MatchState};
#[cfg(target_arch = "wasm32")]
use crate::systems::presentation::layout::mobile_layout;
#[cfg(target_arch = "wasm32")]
use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn browser_mobile_state() {}
#[cfg(target_arch = "wasm32")]
pub(crate) fn browser_mobile_state(
    windows: Query<&Window>,
    cameras: Query<(&Transform, &Projection), With<SpectatorCamera>>,
) {
    let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("game-status"))
    else {
        return;
    };
    if let Some(window) = windows.iter().next() {
        let _ = status.set_attribute(
            "data-mobile-layout",
            if mobile_layout(window.width(), window.height()) {
                "true"
            } else {
                "false"
            },
        );
    }
    for (transform, projection) in &cameras {
        if let Projection::Orthographic(orthographic) = projection {
            let _ = status.set_attribute("data-camera-scale", &orthographic.scale.to_string());
        }
        let _ = status.set_attribute("data-camera-x", &transform.translation.x.to_string());
        let _ = status.set_attribute("data-camera-z", &transform.translation.z.to_string());
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn browser_state() {}
#[cfg(target_arch = "wasm32")]
pub(crate) fn browser_state(
    state: Res<MatchState>,
    inspector: Res<InspectorState>,
    minions: Query<(), With<Minion>>,
    towers: Query<Option<&PlayerControlled>, With<Tower>>,
    deposits: Query<&ElementInventory, With<ElementDeposit>>,
    crown: Query<(), With<DonutVisual>>,
    inventories: Query<(&ElementInventory, Option<&Tower>), Or<(With<Tower>, With<Minion>)>>,
    trails: Query<(), With<Trail>>,
    economy: Res<ElementEconomy>,
    equipped_visuals: Query<(), With<EquippedElementVisual>>,
    projectiles: Query<(), With<Projectile>>,
    order_visuals: Query<&EntityKind, Or<(With<StackPiece>, With<TowerBeacon>)>>,
) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(loading) = document.get_element_by_id("loading") {
        let _ = loading.set_attribute("hidden", "");
    }
    if let Some(status) = document.get_element_by_id("game-status") {
        for (key, value) in [
            ("data-ready", "true".to_string()),
            ("data-minions", minions.iter().count().to_string()),
            ("data-towers", towers.iter().count().to_string()),
            (
                "data-player-towers",
                towers
                    .iter()
                    .filter(|controlled| controlled.is_some())
                    .count()
                    .to_string(),
            ),
            ("data-paused", state.paused.to_string()),
            ("data-speed", state.speed.to_string()),
            ("data-elapsed", state.elapsed.to_string()),
            ("data-inspector", inspector.open.to_string()),
            (
                "data-elements-deposits",
                deposits.iter().count().to_string(),
            ),
            ("data-donut-visuals", crown.iter().count().to_string()),
            (
                "data-elements-remaining",
                deposits
                    .iter()
                    .map(|raw| raw.total() as u64)
                    .sum::<u64>()
                    .to_string(),
            ),
            (
                "data-elements-carried",
                inventories
                    .iter()
                    .filter(|(_, tower)| tower.is_none())
                    .map(|(raw, _)| raw.total() as u64)
                    .sum::<u64>()
                    .to_string(),
            ),
            (
                "data-elements-stock",
                inventories
                    .iter()
                    .filter(|(_, tower)| tower.is_some())
                    .map(|(raw, _)| raw.total() as u64)
                    .sum::<u64>()
                    .to_string(),
            ),
            (
                "data-elements-initial",
                economy
                    .initial
                    .iter()
                    .map(|&count| u64::from(count))
                    .sum::<u64>()
                    .to_string(),
            ),
            (
                "data-elements-consumed",
                economy
                    .consumed
                    .iter()
                    .map(|&count| u64::from(count))
                    .sum::<u64>()
                    .to_string(),
            ),
            ("data-trails", trails.iter().count().to_string()),
            (
                "data-equipped-visuals",
                equipped_visuals.iter().count().to_string(),
            ),
            (
                "data-stack-pieces",
                order_visuals
                    .iter()
                    .filter(|kind| **kind == EntityKind::StackVisual)
                    .count()
                    .to_string(),
            ),
            (
                "data-tower-beacons",
                order_visuals
                    .iter()
                    .filter(|kind| **kind == EntityKind::TowerBeacon)
                    .count()
                    .to_string(),
            ),
            ("data-projectiles", projectiles.iter().count().to_string()),
        ] {
            let _ = status.set_attribute(key, &value);
        }
    }
}
// The page records visibility transitions even while requestAnimationFrame is suspended.
// Consume the resume latch before FixedMain so hidden wall time never advances the match.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn browser_visibility() {}
#[cfg(target_arch = "wasm32")]
pub(crate) fn browser_visibility(mut state: ResMut<MatchState>) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let resumed = document
        .get_element_by_id("game-status")
        .is_some_and(|status| {
            let resumed = status.get_attribute("data-resumed").as_deref() == Some("true");
            if resumed {
                let _ = status.remove_attribute("data-resumed");
            }
            resumed
        });
    state.browser_hidden = document.hidden() || resumed;
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn browser_control_positions() {}
#[cfg(target_arch = "wasm32")]
pub(crate) fn browser_control_positions(
    controls: Query<(&Control, &ComputedNode, &UiGlobalTransform)>,
    hud: Res<HudVisibility>,
    state: Res<MatchState>,
    areas: Query<(), With<TowerInfluenceVisual>>,
    towers: Query<&InfluenceEmitter, With<Tower>>,
) {
    let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("game-status"))
    else {
        return;
    };
    let mut positions = Vec::new();
    for (control, node, transform) in &controls {
        if node.size() == Vec2::ZERO {
            continue;
        }
        let key = match control {
            Control::Pause => "pause",
            Control::Hud => "hud",
            Control::Restart => "restart",
            Control::Speed(speed) if *speed == 1.0 => "1x",
            Control::Speed(speed) if *speed == 2.0 => "2x",
            Control::Speed(speed) if *speed == 4.0 => "4x",
            _ => continue,
        };
        let center = transform.translation * node.inverse_scale_factor();
        positions.push(format!("\"{key}\":[{},{}]", center.x, center.y));
    }
    let _ = status.set_attribute(
        "data-control-positions",
        &format!("{{{}}}", positions.join(",")),
    );
    let _ = status.set_attribute(
        "data-hud-visible",
        if hud.visible { "true" } else { "false" },
    );
    let _ = status.set_attribute(
        "data-pause-action",
        if state.paused { "play" } else { "pause" },
    );
    let _ = status.set_attribute("data-influence-areas", &areas.iter().count().to_string());
    let _ = status.set_attribute(
        "data-max-tower-radius",
        &towers
            .iter()
            .map(|emitter| emitter.radius)
            .fold(0.0_f32, f32::max)
            .to_string(),
    );
}
