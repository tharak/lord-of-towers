use crate::components::{
    Allegiance, ArmState, Attack, DragControl, ElementInventory, ElementStats, EntityKind,
    EquipmentOrder, EquippedElements, Faction, GatheringState, Health, InfluenceEmitter,
    InspectorRoot, InspectorText, LegState, Movement, Projectile, Trail, Vision, VisionBlocker,
};
use crate::resources::{InspectorFilter, InspectorState};
use crate::systems::inspector::filtering::matching_entities;
use crate::types::{Element, FACTION_NAMES, KINDS};
use bevy::prelude::*;
use std::fmt::Write;

// Preserve generic wrappers while removing module paths from each type token.
pub(crate) fn short_type_name(name: &str) -> String {
    let mut result = String::new();
    let mut token = String::new();
    for character in name.chars().chain(std::iter::once(' ')) {
        if character.is_alphanumeric() || character == '_' || character == ':' {
            token.push(character);
        } else {
            result.push_str(token.rsplit("::").next().unwrap_or(&token));
            token.clear();
            result.push(character);
        }
    }
    result.trim_end().to_owned()
}
pub(crate) fn append_gameplay_details(text: &mut String, entity: &EntityRef) {
    if let Some(inventory) = entity.get::<ElementInventory>() {
        let counts = inventory.counts();
        let _ = writeln!(
            text,
            "Elements: {} | Fire {} Water {} Earth {} Air {} Void {}",
            inventory.total(),
            counts[0],
            counts[1],
            counts[2],
            counts[3],
            counts[4]
        );
    }
    if let Some(stats) = entity.get::<ElementStats>() {
        let _ = writeln!(
            text,
            "Birth recipe points (do not scale equipment): Fire {} Water {} Earth {} Air {} Void {}",
            stats.fire, stats.water, stats.earth, stats.air, stats.void
        );
    }
    if let Some(health) = entity.get::<Health>() {
        let _ = writeln!(text, "Health: {:.1}/{:.1}", health.current, health.max);
    }
    if let Some(movement) = entity.get::<Movement>() {
        let _ = writeln!(text, "Movement speed: {:.1}", movement.speed);
    }
    if let Some(attack) = entity.get::<Attack>() {
        let _ = writeln!(
            text,
            "Attack: {:.1} damage | {:.1} range | {:.1}s cooldown",
            attack.damage, attack.range, attack.cooldown
        );
    }
    if let Some(vision) = entity.get::<Vision>() {
        let _ = writeln!(text, "Vision: {:.1}", vision.radius);
    }
    if let Some(blocker) = entity.get::<VisionBlocker>() {
        let _ = writeln!(
            text,
            "Sight blocker: {:?} | half extents ({:.2}, {:.2}) | angle {:.2}",
            blocker.shape, blocker.half_extents.x, blocker.half_extents.y, blocker.angle
        );
    }
    if let Some(state) = entity.get::<LegState>() {
        let _ = writeln!(text, "Leg animation state: {state:?}");
    }
    if let Some(state) = entity.get::<ArmState>() {
        let _ = writeln!(
            text,
            "Arms: carrying={} shooting={}",
            state.carrying, state.shooting
        );
    }
    if let Some(drag) = entity.get::<DragControl>() {
        let _ = writeln!(
            text,
            "Drag: {:?} target ({:.1}, {:.1})",
            drag.phase, drag.target.x, drag.target.y
        );
    }
    if let Some(state) = entity.get::<GatheringState>() {
        let _ = writeln!(text, "Gathering: {state:?}");
    }
    if let Some(trail) = entity.get::<Trail>() {
        let _ = writeln!(
            text,
            "Trail: trip {} | sequence {} | {:.1}s remaining",
            trail.trip, trail.sequence, trail.remaining
        );
    }
    if let Some(emitter) = entity.get::<InfluenceEmitter>() {
        let _ = writeln!(text, "Influence radius: {:.1}", emitter.radius);
    }
    if let Some(gear) = entity.get::<EquippedElements>() {
        for (slot, element) in gear.slots.iter().enumerate() {
            let _ = writeln!(
                text,
                "Equipment slot {}: {}",
                slot + 1,
                element.map_or("Empty", Element::name)
            );
        }
    }
    if let Some(order) = entity.get::<EquipmentOrder>() {
        let _ = writeln!(
            text,
            "Equipment order: {:?} | tower {:?} | source position ({:.1}, {:.1}) | recipient minion {:?}",
            order.action,
            order.tower,
            order.source_position.x,
            order.source_position.y,
            order.minion
        );
        let _ = writeln!(
            text,
            "Reserved units: {}",
            entity
                .get::<ElementInventory>()
                .map_or(0, ElementInventory::total)
        );
    }
    if let Some(projectile) = entity.get::<Projectile>() {
        let _ = writeln!(
            text,
            "Projectile: target {:?} | {:.1} damage | {:.1} speed | {:.1}s remaining",
            projectile.target, projectile.damage, projectile.speed, projectile.remaining
        );
    }
}
pub(crate) fn refresh(world: &mut World) {
    let open = world.resource::<InspectorState>().open;
    let viewport = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map(|window| (window.width(), window.height()));
    let mobile = viewport.is_some_and(|(width, height)| width < 760.0 || height < 540.0);
    let mut roots = world.query_filtered::<&mut Node, With<InspectorRoot>>();
    for mut node in roots.iter_mut(world) {
        node.display = if open { Display::Flex } else { Display::None };
        if let Some((width, height)) = viewport {
            node.top = px(if mobile { 86 } else { 90 });
            node.right = px(if mobile { 8 } else { 12 });
            node.width = px(if mobile {
                (width - 16.0).max(1.0)
            } else {
                520.0
            });
            node.max_height = if mobile {
                px((height - 202.0).max(80.0))
            } else {
                Val::Auto
            };
            node.padding = UiRect::all(px(if mobile { 8 } else { 12 }));
            node.overflow = if mobile {
                Overflow::clip()
            } else {
                Overflow::visible()
            };
        }
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
        "LIVE ECS -- {total} entities, {} matching\nKind: {kind_name} | Faction: {faction_name}\nEngine entities: {} | Entity {}/{}\n",
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
            "\n{id:?} -- {name}\nKind: {:?}\nFaction: {}",
            entity.get::<EntityKind>(),
            faction.map_or("None", |f| FACTION_NAMES[f.index])
        );
        append_gameplay_details(&mut text, &entity);
        let mut components: Vec<_> = entity
            .archetype()
            .components()
            .iter()
            .filter_map(|id| world.components().get_info(*id))
            .map(|info| info.name().to_string())
            .collect();
        components.sort();
        let component_count = if mobile { 5 } else { 16 };
        let pages = components.len().div_ceil(component_count).max(1);
        let component_page = component_page % pages;
        let _ = writeln!(
            text,
            "\nComponents: {} (page {}/{pages})",
            components.len(),
            component_page + 1
        );
        for name in components
            .iter()
            .skip(component_page * component_count)
            .take(component_count)
        {
            // Short Rust type names fit the panel; names are obtained from ECS metadata.
            let _ = writeln!(text, "- {}", short_type_name(name));
        }
    } else {
        text.push_str("\nNo matching live entities.");
    }
    if mobile {
        // Keep the live view readable inside the bounded panel. Components remain paged.
        let (width, height) = viewport.unwrap();
        let columns = ((width - 36.0) / 7.5).max(12.0) as usize;
        let lines = ((height - 350.0) / 17.0).max(3.0) as usize;
        text = text
            .lines()
            .filter(|line| !line.is_empty())
            .take(lines)
            .map(|line| line.chars().take(columns).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
    }
    let mut details = world.query_filtered::<&mut Text, With<InspectorText>>();
    for mut target in details.iter_mut(world) {
        target.0.clone_from(&text);
    }
    let mut filter = world.resource_mut::<InspectorFilter>();
    filter.page = page;
    filter.next_refresh = now + 0.25;
}
