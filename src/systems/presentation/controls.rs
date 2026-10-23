use crate::components::{Control, HudRoot, HudToggleText, PauseIcon};
use crate::resources::{HudVisibility, MatchState};
use bevy::prelude::*;

pub(crate) fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<MatchState>) {
    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }
    for (key, speed) in [
        (KeyCode::Digit1, 1.0),
        (KeyCode::Digit2, 2.0),
        (KeyCode::Digit3, 4.0),
    ] {
        if keys.just_pressed(key) {
            state.speed = speed;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.restart_requested = true;
    }
}
pub(crate) fn button_controls(
    mut buttons: Query<(Ref<Interaction>, &Control, &mut BackgroundColor)>,
    mut state: ResMut<MatchState>,
    mut hud: ResMut<HudVisibility>,
) {
    // Act only on a new press while updating button colors every frame.
    for (interaction, control, mut color) in &mut buttons {
        if *interaction == Interaction::Pressed && interaction.is_changed() {
            match control {
                Control::Pause => state.paused = !state.paused,
                Control::Speed(speed) => state.speed = *speed,
                Control::Restart => state.restart_requested = true,
                Control::Hud => hud.visible = !hud.visible,
            }
        }
        color.0 = if *interaction == Interaction::None {
            Color::srgb(0.13, 0.17, 0.23)
        } else {
            Color::srgb(0.24, 0.32, 0.43)
        };
    }
}
pub(crate) fn sync_control_state(
    state: Res<MatchState>,
    hud: Res<HudVisibility>,
    mut roots: Query<&mut Node, (With<HudRoot>, Without<PauseIcon>)>,
    mut icons: Query<(&PauseIcon, &mut Node), Without<HudRoot>>,
    mut hud_labels: Query<&mut Text, With<HudToggleText>>,
    mut buttons: Query<(
        &Control,
        &mut Name,
        Option<&mut bevy::a11y::AccessibilityNode>,
    )>,
) {
    for mut text in &mut hud_labels {
        text.0 = String::from(if hud.visible { "Hide HUD" } else { "Show HUD" });
    }
    for mut node in &mut roots {
        node.display = if hud.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (icon, mut node) in &mut icons {
        node.display = if icon.0 == state.paused {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (control, mut name, accessible) in &mut buttons {
        match control {
            Control::Pause => *name = Name::new(if state.paused { "Play" } else { "Pause" }),
            Control::Hud => *name = Name::new(if hud.visible { "Hide HUD" } else { "Show HUD" }),
            _ => {}
        }
        if let Some(mut accessible) = accessible {
            accessible.set_label(name.as_str());
        }
    }
}
