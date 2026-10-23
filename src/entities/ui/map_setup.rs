//! Persistent map settings, seed controls, preview/start actions, and a new-map launcher.
use crate::components::{EntityKind, MapSetupAction, MapSetupRoot, MapSetupText, MapStartupRoot};
use crate::resources::MapCreation;
use crate::types::map_ui::MapTextRole;
use bevy::prelude::*;

fn label(parent: &mut ChildSpawnerCommands, text: &str, size: f32) {
    parent.spawn((
        Name::new(text.to_owned()),
        EntityKind::Ui,
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.88, 0.9)),
    ));
}
fn button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: MapSetupAction,
    role: Option<MapTextRole>,
) {
    parent
        .spawn((
            Name::new(text.to_owned()),
            EntityKind::Ui,
            Button,
            action,
            Node {
                min_height: px(38),
                flex_shrink: 0.0,
                padding: UiRect::all(px(9)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.22, 0.25)),
        ))
        .with_children(|parent| {
            let mut text = parent.spawn((
                Name::new("Map option label"),
                EntityKind::Ui,
                Text::new(text),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
            ));
            if let Some(role) = role {
                text.insert(MapSetupText(role));
            }
        });
}
pub(crate) fn spawn_map_setup(mut commands: Commands, map: Res<MapCreation>) {
    if !map.options_enabled {
        spawn_startup_status(&mut commands, &map.message);
        return;
    }
    commands
        .spawn((
            Name::new("Map creation"),
            EntityKind::Ui,
            MapSetupRoot,
            Interaction::None,
            Node {
                position_type: PositionType::Absolute,
                left: px(10),
                top: px(10),
                bottom: px(10),
                width: px(330),
                max_width: percent(94),
                padding: UiRect::all(px(14)),
                row_gap: px(7),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            GlobalZIndex(40),
            BackgroundColor(Color::srgba(0.025, 0.045, 0.055, 0.97)),
        ))
        .with_children(|parent| {
            label(parent, "CREATE YOUR MAP", 22.0);
            label(parent, "Choose a preset, then customize each layer.", 12.0);
            button(
                parent,
                "Show preview / hide settings",
                MapSetupAction::TogglePanel,
                None,
            );
            for (label, action, role) in [
                ("Preset", MapSetupAction::Preset, MapTextRole::Preset),
                ("Relief", MapSetupAction::Relief, MapTextRole::Relief),
                ("Water", MapSetupAction::Water, MapTextRole::Water),
                (
                    "Vegetation",
                    MapSetupAction::Vegetation,
                    MapTextRole::Vegetation,
                ),
                (
                    "Settlements",
                    MapSetupAction::Settlements,
                    MapTextRole::Settlements,
                ),
                ("Roads", MapSetupAction::Roads, MapTextRole::Roads),
                ("Climate", MapSetupAction::Climate, MapTextRole::Climate),
            ] {
                button(parent, label, action, Some(role));
            }
            parent.spawn((
                Name::new("Map seed"),
                EntityKind::Ui,
                MapSetupText(MapTextRole::Seed),
                Text::new("Seed: 42"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
            ));
            parent
                .spawn((
                    Name::new("Seed controls"),
                    EntityKind::Ui,
                    Node {
                        column_gap: px(6),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    button(row, "− 1", MapSetupAction::SeedPrevious, None);
                    button(row, "+ 1", MapSetupAction::SeedNext, None);
                    button(row, "Shuffle seed", MapSetupAction::SeedShuffle, None);
                });
            button(parent, "Generate preview", MapSetupAction::Generate, None);
            button(parent, "Start match", MapSetupAction::Start, None);
            parent.spawn((
                Name::new("Map generation status"),
                EntityKind::Ui,
                MapSetupText(MapTextRole::Status),
                Text::new("Choose your map, then generate a preview."),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
            ));
            label(
                parent,
                "Decorative terrain · movement and combat stay the same.",
                11.0,
            );
            label(parent, "Maps powered by Magna Urbis · GPLv3", 11.0);
        });
    commands
        .spawn((
            Name::new("Map settings launcher"),
            EntityKind::Ui,
            Button,
            MapSetupAction::NewMap,
            Node {
                position_type: PositionType::Absolute,
                right: px(112),
                top: px(8),
                width: px(110),
                height: px(44),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(41),
            BackgroundColor(Color::srgb(0.15, 0.22, 0.25)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Map settings launcher label"),
                EntityKind::Ui,
                MapSetupText(MapTextRole::Launcher),
                Text::new("Map settings"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
            ));
        });
}

fn spawn_startup_status(commands: &mut Commands, message: &str) {
    commands
        .spawn((
            Name::new("Battlefield startup"),
            EntityKind::Ui,
            MapStartupRoot,
            Node {
                position_type: PositionType::Absolute,
                left: percent(5),
                right: percent(5),
                top: percent(40),
                padding: UiRect::all(px(16)),
                row_gap: px(12),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(40),
            BackgroundColor(Color::srgba(0.025, 0.045, 0.055, 0.97)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Battlefield startup status"),
                EntityKind::Ui,
                MapSetupText(MapTextRole::Status),
                Text::new(message),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
            ));
            button(parent, "Retry map", MapSetupAction::Generate, None);
        });
}
