//! A bounded view into the live ECS world, never an entity registry.
use crate::model::{Allegiance, EntityKind, FACTION_NAMES, Faction};
use bevy::prelude::*;
use std::fmt::Write;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorState>()
            .init_resource::<InspectorFilter>()
            .add_systems(Startup, setup)
            .add_systems(Update, (controls, refresh).chain());
    }
}
#[derive(Resource, Default)]
pub struct InspectorState {
    pub open: bool,
}
#[derive(Resource, Default)]
struct InspectorFilter {
    kind: usize,
    faction: usize,
    engine: bool,
    page: usize,
    component_page: usize,
    next_refresh: f64,
}
#[derive(Component)]
struct InspectorRoot;
#[derive(Component)]
struct InspectorText;
#[derive(Component, Clone, Copy)]
enum InspectorAction {
    Kind,
    Faction,
    Engine,
    Previous,
    Next,
    Components,
    Close,
}
const KINDS: [EntityKind; 7] = [
    EntityKind::Faction,
    EntityKind::Tower,
    EntityKind::Minion,
    EntityKind::Battlefield,
    EntityKind::Camera,
    EntityKind::Light,
    EntityKind::Ui,
];

fn setup(mut commands: Commands) {
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
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    actions: Query<(&Interaction, &InspectorAction), Changed<Interaction>>,
    mut state: ResMut<InspectorState>,
    mut filter: ResMut<InspectorFilter>,
) {
    if keys.just_pressed(KeyCode::F3) {
        state.open = !state.open;
        filter.next_refresh = 0.0;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            InspectorAction::Kind => {
                filter.kind = (filter.kind + 1) % (KINDS.len() + 1);
                filter.page = 0;
            }
            InspectorAction::Faction => {
                filter.faction = (filter.faction + 1) % 6;
                filter.page = 0;
            }
            InspectorAction::Engine => {
                filter.engine = !filter.engine;
                filter.page = 0;
            }
            InspectorAction::Previous => {
                filter.page = filter.page.saturating_sub(1);
                filter.component_page = 0;
            }
            InspectorAction::Next => {
                filter.page += 1;
                filter.component_page = 0;
            }
            InspectorAction::Components => filter.component_page += 1,
            InspectorAction::Close => state.open = false,
        }
        filter.next_refresh = 0.0;
    }
}

fn matching_entities(
    world: &mut World,
    kind: usize,
    faction: usize,
    engine: bool,
) -> (usize, Vec<Entity>) {
    let mut query = world.query::<EntityRef>();
    let mut total = 0;
    let mut matched = Vec::new();
    for entity in query.iter(world) {
        total += 1;
        let entity_kind = entity.get::<EntityKind>();
        if entity_kind.is_none() && !engine {
            continue;
        }
        if kind != 0 && entity_kind != Some(&KINDS[kind - 1]) {
            continue;
        }
        if faction != 0 {
            let own_faction = entity.get::<Faction>().or_else(|| {
                entity
                    .get::<Allegiance>()
                    .and_then(|a| world.get::<Faction>(a.0))
            });
            if own_faction.is_none_or(|f| f.index != faction - 1) {
                continue;
            }
        }
        matched.push(entity.id());
    }
    matched.sort_unstable();
    (total, matched)
}

fn refresh(world: &mut World) {
    let open = world.resource::<InspectorState>().open;
    let mut roots = world.query_filtered::<&mut Node, With<InspectorRoot>>();
    for mut node in roots.iter_mut(world) {
        node.display = if open { Display::Flex } else { Display::None };
    }
    if !open {
        return;
    }
    let now = world.resource::<Time>().elapsed_secs_f64();
    let filter = world.resource::<InspectorFilter>();
    if now < filter.next_refresh {
        return;
    }
    let (kind, faction, engine, page, component_page) = (
        filter.kind,
        filter.faction,
        filter.engine,
        filter.page,
        filter.component_page,
    );
    let (total, entities) = matching_entities(world, kind, faction, engine);
    let page = page.min(entities.len().saturating_sub(1));
    let kind_name = if kind == 0 {
        "All".to_owned()
    } else {
        format!("{:?}", KINDS[kind - 1])
    };
    let faction_name = if faction == 0 {
        "All"
    } else {
        FACTION_NAMES[faction - 1]
    };
    let mut text = format!(
        "LIVE ECS — {total} entities, {} matching\nKind: {kind_name} | Faction: {faction_name}\nEngine entities: {} | Entity {}/{}\n",
        entities.len(),
        if engine { "included" } else { "hidden" },
        if entities.is_empty() { 0 } else { page + 1 },
        entities.len()
    );
    if let Some(id) = entities.get(page) {
        let entity = world.entity(*id);
        let name = entity.get::<Name>().map_or("Unnamed", |name| name.as_str());
        let faction = entity.get::<Faction>().or_else(|| {
            entity
                .get::<Allegiance>()
                .and_then(|a| world.get::<Faction>(a.0))
        });
        let _ = writeln!(
            text,
            "\n{id:?} — {name}\nKind: {:?}\nFaction: {}",
            entity.get::<EntityKind>(),
            faction.map_or("None", |f| FACTION_NAMES[f.index])
        );
        let mut components: Vec<_> = entity
            .archetype()
            .components()
            .iter()
            .filter_map(|id| world.components().get_info(*id))
            .map(|info| info.name().to_string())
            .collect();
        components.sort();
        let pages = components.len().div_ceil(16).max(1);
        let component_page = component_page % pages;
        let _ = writeln!(
            text,
            "\nComponents: {} (page {}/{pages})",
            components.len(),
            component_page + 1
        );
        for name in components.iter().skip(component_page * 16).take(16) {
            // Short Rust type names fit the panel; names are obtained from ECS metadata.
            let _ = writeln!(text, "• {}", name.rsplit("::").next().unwrap_or(name));
        }
    } else {
        text.push_str("\nNo matching live entities.");
    }
    let mut details = world.query_filtered::<&mut Text, With<InspectorText>>();
    for mut target in details.iter_mut(world) {
        target.0.clone_from(&text);
    }
    let mut filter = world.resource_mut::<InspectorFilter>();
    filter.page = page;
    filter.next_refresh = now + 0.25;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filters_follow_live_world_and_never_retain_despawned_entities() {
        let mut world = World::new();
        let faction = world
            .spawn((
                Faction { index: 2 },
                EntityKind::Faction,
                Name::new("Faction"),
            ))
            .id();
        let minion = world
            .spawn((Allegiance(faction), EntityKind::Minion, Name::new("Minion")))
            .id();
        world.spawn_empty();
        assert_eq!(matching_entities(&mut world, 0, 0, false).1.len(), 2);
        assert_eq!(matching_entities(&mut world, 0, 0, true).1.len(), 3);
        assert_eq!(matching_entities(&mut world, 3, 3, false).1, vec![minion]);
        world.despawn(minion);
        assert!(matching_entities(&mut world, 3, 3, false).1.is_empty());
    }
}
