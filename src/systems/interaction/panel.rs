use crate::components::{
    Allegiance, ContextAction, ContextPanel, ElementInventory, EquipmentOrder, EquippedElements,
    Health, PlayerControlled, Position, Tower,
};
use crate::entities::ui::context_panel::spawn_context_panel;
use crate::resources::{PanelCache, PlayerCommands, PlayerSelection};
use crate::types::{Element, PlayerCommand, Row};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub(crate) fn row(label: impl Into<String>, action: Option<ContextAction>) -> Row {
    Row {
        label: label.into(),
        action,
        color: Color::srgb(0.15, 0.19, 0.25),
    }
}
// Queued births also claim stock while simulation is paused or between fixed ticks.
pub(crate) fn recipe_stock(
    inventory: &ElementInventory,
    tower: Entity,
    commands: &PlayerCommands,
) -> [u32; 5] {
    let mut stock = inventory.counts();
    for command in &commands.0 {
        if let PlayerCommand::CreateMinion {
            tower: source,
            counts,
        } = command
            && *source == tower
        {
            for (available, selected) in stock.iter_mut().zip(counts) {
                *available = available.saturating_sub(*selected);
            }
        }
    }
    stock
}
pub(crate) fn context_panel(world: &mut World) {
    let selected = world.resource::<PlayerSelection>().entity;
    let mut recipe = world.resource::<PlayerSelection>().recipe;
    let owner = world
        .query_filtered::<&Allegiance, (With<Tower>, With<PlayerControlled>)>()
        .iter(world)
        .next()
        .copied();
    let valid = selected.filter(|e| {
        world.get::<Health>(*e).is_some_and(|h| h.current > 0.0)
            && world.get::<Allegiance>(*e).copied() == owner
            && owner.is_some()
    });
    if selected != valid {
        world.resource_mut::<PlayerSelection>().entity = valid;
    }
    let mut element_rows = Vec::new();
    let mut anchor = Vec2::ZERO;
    let mut viewport = Vec2::new(320.0, 640.0);
    if let Some(entity) = valid {
        let position = world.get::<Position>(entity).unwrap().0;
        if let Some(window) = world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .iter(world)
            .next()
        {
            viewport = Vec2::new(window.width(), window.height());
        }
        if let Some((camera, transform)) = world
            .query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>()
            .iter(world)
            .next()
        {
            anchor = camera
                .world_to_viewport(transform, Vec3::new(position.x, 1.0, position.y))
                .unwrap_or(viewport * 0.5);
        }
        if world.get::<Tower>(entity).is_some() {
            let stock = world
                .get::<ElementInventory>(entity)
                .map_or([0; 5], |inventory| {
                    recipe_stock(inventory, entity, world.resource::<PlayerCommands>())
                });
            for (selected, available) in recipe.iter_mut().zip(stock) {
                *selected = (*selected).min(available);
            }
            world.resource_mut::<PlayerSelection>().recipe = recipe;
            let choices = Element::ALL
                .into_iter()
                .map(|element| {
                    let index = element as usize;
                    let mut choice = row(
                        format!("{}\n{}", element.name(), recipe[index]),
                        (recipe[index] < stock[index]).then_some(ContextAction::Recipe {
                            tower: entity,
                            element,
                        }),
                    );
                    choice.color = element.color();
                    (choice, recipe[index] > 0)
                })
                .collect();
            element_rows.push(choices);
        } else {
            let gear = world
                .get::<EquippedElements>(entity)
                .copied()
                .unwrap_or_default();
            let pending = world
                .query::<(&EquipmentOrder, &ElementInventory)>()
                .iter(world)
                .find(|(order, _)| order.minion == entity)
                .map(|(order, stock)| (*order, stock.counts()));
            let nearest = world.query_filtered::<(Entity, &Position, &Allegiance, &Health, &ElementInventory), With<Tower>>().iter(world)
                .filter(|(_, _, a, h, _)| Some(**a) == owner && h.current > 0.0)
                .min_by(|a, b| a.1.0.distance_squared(position).total_cmp(&b.1.0.distance_squared(position)))
                .map(|(e, _, _, _, stock)| (e, stock.counts()));
            for slot in 0..3 {
                let mut choices = Vec::new();
                for element in Element::ALL {
                    let equipped = gear.slots[slot] == Some(element);
                    let action = nearest.and_then(|(tower, stock)| {
                        let refunded = pending
                            .filter(|(order, _)| order.tower == tower)
                            .map_or(0, |(_, stock)| stock[element as usize]);
                        let command = if equipped {
                            PlayerCommand::ReturnEquipment {
                                minion: entity,
                                tower,
                                slot,
                            }
                        } else if stock[element as usize] > 0 || refunded > 0 {
                            PlayerCommand::Equip {
                                minion: entity,
                                tower,
                                slot,
                                element,
                            }
                        } else {
                            return None;
                        };
                        Some(ContextAction::Equipment {
                            command,
                            cancel: pending.is_some(),
                        })
                    });
                    let mut choice = row(element.name(), action);
                    choice.color = element.color();
                    choices.push((choice, equipped));
                }
                element_rows.push(choices);
            }
        }
    }
    let signature = format!(
        "{valid:?}:{recipe:?}:{}",
        element_rows
            .iter()
            .flatten()
            .map(|(r, selected)| format!("{}:{:?}:{selected}", r.label, r.action))
            .collect::<String>()
    );
    let width = 252.0_f32.min((viewport.x - 16.0).max(1.0));
    let height = 12.0 + element_rows.len() as f32 * 43.0 - 3.0;
    let left =
        (anchor.x + 20.0).clamp(8.0_f32.min(viewport.x), (viewport.x - width - 8.0).max(8.0));
    let top = (anchor.y - height * 0.5).clamp(
        8.0_f32.min(viewport.y),
        (viewport.y - height - 8.0).max(8.0),
    );
    if world.resource::<PanelCache>().0 == signature {
        for mut node in world
            .query_filtered::<&mut Node, With<ContextPanel>>()
            .iter_mut(world)
        {
            node.left = px(left);
            node.top = px(top);
            node.width = px(width);
            node.max_height = px(height);
        }
        return;
    }
    world.resource_mut::<PanelCache>().0 = signature;
    let old: Vec<_> = world
        .query_filtered::<Entity, With<ContextPanel>>()
        .iter(world)
        .collect();
    for entity in old {
        world.despawn(entity);
    }
    if valid.is_none() {
        return;
    }
    spawn_context_panel(world, left, top, width, height, element_rows);
}
