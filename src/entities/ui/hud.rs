//! Constructs hud UI entities and their children.
//! Behavior and lifecycle decisions live in interaction, HUD, or inspector systems.
use crate::components::{
    Control, ControlBar, DesktopDetail, EntityKind, FactionText, HudRoot, HudTitle, HudToggleText,
    PauseIcon, StatusText,
};
use crate::systems::interaction::panel::row;
use crate::types::{FACTION_NAMES, faction_color};
use bevy::prelude::*;

pub(crate) fn spawn_pause_icon(button: &mut ChildSpawnerCommands) {
    for paused in [false, true] {
        button
            .spawn((
                Name::new(if paused { "Play icon" } else { "Pause icon" }),
                EntityKind::Ui,
                PauseIcon(paused),
                Node {
                    width: px(18),
                    height: px(20),
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ))
            .with_children(|icon| {
                if paused {
                    for row in 0..10 {
                        let width = (10 - (row * 2_i32 - 9).abs()) as f32 * 1.8;
                        icon.spawn((
                            Name::new("Play icon strip"),
                            EntityKind::Ui,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(2),
                                top: px(row * 2),
                                width: px(width),
                                height: px(2),
                                ..default()
                            },
                            BackgroundColor(Color::WHITE),
                        ));
                    }
                } else {
                    for left in [2, 11] {
                        icon.spawn((
                            Name::new("Pause icon bar"),
                            EntityKind::Ui,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(left),
                                width: px(5),
                                height: px(20),
                                ..default()
                            },
                            BackgroundColor(Color::WHITE),
                        ));
                    }
                }
            });
    }
}
pub(crate) fn spawn_hud(commands: &mut Commands) {
    commands
        .spawn((
            Name::new("Match information"),
            HudRoot,
            EntityKind::Ui,
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(20),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(14)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.035, 0.055, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Game title"),
                HudTitle,
                EntityKind::Ui,
                Text::new("LORD OF TOWERS"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
            ));
            parent.spawn((
                Name::new("Match status"),
                EntityKind::Ui,
                StatusText,
                Text::new("Starting simulation..."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
            ));
            for (i, name) in FACTION_NAMES.iter().enumerate() {
                parent.spawn((
                    Name::new(format!("{name} statistics")),
                    EntityKind::Ui,
                    FactionText(i),
                    DesktopDetail,
                    Text::new(*name),
                    TextColor(faction_color(i)),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                ));
            }
            parent.spawn((
                Name::new("Camera instructions"),
                DesktopDetail,
                EntityKind::Ui,
                Text::new(
                    "Pan: WASD / arrows   Zoom: + / -\nSpace: pause   1/2/3: speed   R: restart\nSelect your tower or minions to manage equipment",
                ),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.7, 0.8)),
            ));
            parent.spawn((
                Name::new("Build identifier"),
                DesktopDetail,
                EntityKind::Ui,
                Text::new(format!(
                    "Build {}",
                    option_env!("BUILD_COMMIT")
                        .unwrap_or("development")
                        .chars()
                        .take(7)
                        .collect::<String>()
                )),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
            ));
        });
    commands
        .spawn((
            Name::new("Spectator controls"),
            ControlBar,
            EntityKind::Ui,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(20),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8),
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (label, control) in [
                ("Pause", Control::Pause),
                ("1x", Control::Speed(1.0)),
                ("2x", Control::Speed(2.0)),
                ("4x", Control::Speed(4.0)),
                ("Restart", Control::Restart),
            ] {
                parent
                    .spawn((
                        Name::new(label),
                        EntityKind::Ui,
                        Button,
                        control,
                        Node {
                            min_width: px(44),
                            height: px(44),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::axes(px(14), px(0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.13, 0.17, 0.23)),
                    ))
                    .with_children(|button| {
                        if matches!(control, Control::Pause) {
                            spawn_pause_icon(button);
                            return;
                        }
                        button.spawn((
                            Name::new(format!("{label} label")),
                            EntityKind::Ui,
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                        ));
                    });
            }
        });
    commands
        .spawn((
            Name::new("Hide HUD"),
            EntityKind::Ui,
            Button,
            Control::Hud,
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                right: px(8),
                width: px(96),
                height: px(44),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(10),
            BackgroundColor(Color::srgb(0.13, 0.17, 0.23)),
        ))
        .with_children(|button| {
            button.spawn((
                Name::new("HUD visibility label"),
                EntityKind::Ui,
                HudToggleText,
                Text::new("Hide HUD"),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
            ));
        });
}
