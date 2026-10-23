use crate::components::{Control, ControlBar, DesktopDetail, HudRoot, HudTitle};
use bevy::prelude::*;

pub(crate) fn mobile_layout(width: f32, height: f32) -> bool {
    width < 760.0 || height < 540.0
}
pub(crate) fn responsive_layout(
    windows: Query<&Window>,
    mut previous_size: Local<Option<Vec2>>,
    mut nodes: Query<(
        &mut Node,
        Option<&HudRoot>,
        Option<&ControlBar>,
        Option<&Control>,
        Option<&DesktopDetail>,
    )>,
    mut titles: Query<&mut TextFont, With<HudTitle>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    if *previous_size == Some(size) {
        return;
    }
    *previous_size = Some(size);
    let mobile = mobile_layout(window.width(), window.height());
    for (mut node, hud, bar, control, detail) in &mut nodes {
        if hud.is_some() {
            node.top = px(if mobile { 8 } else { 18 });
            node.left = px(if mobile { 8 } else { 20 });
            node.padding = UiRect::all(px(if mobile { 8 } else { 14 }));
            node.max_width = if mobile {
                px((window.width() - 128.0).max(1.0))
            } else {
                Val::Auto
            };
        }
        if detail.is_some() {
            node.display = if mobile { Display::None } else { Display::Flex };
        }
        if bar.is_some() {
            node.left = px(8);
            node.right = px(8);
            node.justify_content = JustifyContent::Center;
            node.bottom = px(if mobile { 12 } else { 20 });
            node.column_gap = px(if mobile { 6 } else { 8 });
            node.row_gap = px(if mobile { 6 } else { 8 });
        }
        if control.is_some_and(|control| !matches!(control, Control::Hud)) {
            node.width = if mobile {
                px(((window.width() - 34.0) / 4.0).max(44.0))
            } else {
                Val::Auto
            };
            node.padding = UiRect::axes(px(if mobile { 0 } else { 14 }), px(0));
            node.flex_shrink = 0.0;
        }
    }
    for mut font in &mut titles {
        font.font_size = FontSize::Px(if mobile { 18.0 } else { 24.0 });
    }
}
