//! Constructs context panel UI entities and their children.
//! Behavior and lifecycle decisions live in interaction, HUD, or inspector systems.
use crate::components::{ContextPanel, EntityKind};
use crate::types::Row;
use bevy::prelude::*;

pub(crate) fn spawn_context_panel(
    world: &mut World,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    element_rows: Vec<Vec<(Row, bool)>>,
) {
    world
        .spawn((
            Name::new("Selection context panel"),
            ContextPanel,
            EntityKind::Ui,
            Interaction::None,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(width),
                max_height: px(height),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(6)),
                row_gap: px(3),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            GlobalZIndex(30),
            BackgroundColor(Color::srgba(0.035, 0.045, 0.065, 0.97)),
        ))
        .with_children(|panel| {
            for (slot, choices) in element_rows.into_iter().enumerate() {
                panel
                    .spawn((
                        EntityKind::Ui,
                        Name::new(format!("Element row {}", slot + 1)),
                        Node {
                            width: percent(100),
                            min_height: px(40),
                            flex_shrink: 0.0,
                            column_gap: px(3),
                            ..default()
                        },
                    ))
                    .with_children(|strip| {
                        for (choice, equipped) in choices {
                            let enabled = choice.action.is_some();
                            let mut button = strip.spawn((
                                EntityKind::Ui,
                                Name::new(format!("Element row {} {}", slot + 1, choice.label)),
                                Node {
                                    flex_basis: px(0),
                                    flex_grow: 1.0,
                                    min_width: px(0),
                                    min_height: px(40),
                                    border: UiRect::all(px(2)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BorderColor::all(if equipped { Color::WHITE } else { Color::NONE }),
                                BackgroundColor(if enabled {
                                    choice.color
                                } else {
                                    choice.color.with_alpha(0.22)
                                }),
                            ));
                            if let Some(action) = choice.action {
                                button.insert((Button, action));
                            }
                            button.with_children(|button| {
                                button.spawn((
                                    EntityKind::Ui,
                                    Name::new("Element choice label"),
                                    Text::new(choice.label),
                                    TextFont {
                                        font_size: FontSize::Px(11.0),
                                        ..default()
                                    },
                                    TextColor(if enabled {
                                        Color::WHITE
                                    } else {
                                        Color::srgb(0.55, 0.58, 0.62)
                                    }),
                                ));
                            });
                        }
                    });
            }
        });
}
