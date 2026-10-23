//! Constructs inspector UI entities and their children.
//! Behavior and lifecycle decisions live in interaction, HUD, or inspector systems.
use crate::components::{EntityKind, InspectorAction, InspectorRoot, InspectorText};
use bevy::prelude::*;

pub(crate) fn spawn_inspector_ui(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Entity inspector"),
            EntityKind::Ui,
            InspectorRoot,
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(90),
                width: px(520),
                max_width: percent(95),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.035, 0.055, 0.97)),
            GlobalZIndex(20),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Inspector controls"),
                    EntityKind::Ui,
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(5),
                        row_gap: px(5),
                        ..default()
                    },
                ))
                .with_children(|bar| {
                    for (label, action) in [
                        ("Kind", InspectorAction::Kind),
                        ("Faction", InspectorAction::Faction),
                        ("Engine", InspectorAction::Engine),
                        ("Previous", InspectorAction::Previous),
                        ("Next", InspectorAction::Next),
                        ("Components", InspectorAction::Components),
                        ("Close", InspectorAction::Close),
                    ] {
                        bar.spawn((
                            Name::new(format!("Inspector {label}")),
                            EntityKind::Ui,
                            Button,
                            action,
                            Node {
                                min_height: px(44),
                                min_width: px(44),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                padding: UiRect::axes(px(8), px(5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.2, 0.28)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Name::new(format!("Inspector {label} label")),
                                EntityKind::Ui,
                                Text::new(label),
                                TextFont {
                                    font_size: FontSize::Px(14.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
            parent.spawn((
                Name::new("Inspector live details"),
                EntityKind::Ui,
                InspectorText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
