//! Converts upstream fields and geometry into our bounded, primitive-friendly map data.
use super::{
    settlements::{RoadStroke, base_positions},
    terrain::{RESOLUTION, Surface, WIDTH, raster_line},
};
use crate::types::{
    map_data::{MapData, MapProp, MapPropKind},
    map_settings::{MapClimate, MapDensity, MapGenerationRequest},
};
use bevy::prelude::*;

pub(super) fn convert(
    surface: Surface,
    roads: Vec<RoadStroke>,
    mut props: Vec<MapProp>,
    settlements: usize,
    request: &MapGenerationRequest,
) -> MapData {
    let min = surface.height.iter().copied().fold(f32::INFINITY, f32::min);
    let max = surface
        .height
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let range = (max - min).max(1.0);
    let mut colors = Vec::with_capacity(RESOLUTION * RESOLUTION);
    let mut occupied = vec![false; RESOLUTION * RESOLUTION];
    for y in 0..RESOLUTION {
        for x in 0..RESOLUTION {
            let index = y * RESOLUTION + x;
            let h = (surface.height[index] - min) / range;
            let dx = surface.height[y * RESOLUTION + (x + 1).min(RESOLUTION - 1)]
                - surface.height[y * RESOLUTION + x.saturating_sub(1)];
            let dy = surface.height[(y + 1).min(RESOLUTION - 1) * RESOLUTION + x]
                - surface.height[y.saturating_sub(1) * RESOLUTION + x];
            let shade = (0.88 - dx / range * 1.5 + dy / range).clamp(0.55, 1.1);
            let (low, high) = match request.settings.climate {
                MapClimate::Temperate => (Vec3::new(0.16, 0.27, 0.13), Vec3::new(0.38, 0.36, 0.29)),
                MapClimate::Arid => (Vec3::new(0.48, 0.33, 0.16), Vec3::new(0.64, 0.52, 0.32)),
                MapClimate::Snow => (Vec3::new(0.35, 0.43, 0.40), Vec3::new(0.78, 0.84, 0.87)),
            };
            let color = if surface.water[index] > 0 {
                Vec3::new(0.035, 0.21, 0.31)
            } else {
                low.lerp(high, h) * shade
            };
            colors.push(color.to_array());
            occupied[index] = surface.water[index] > 0;
        }
    }
    for road in roads {
        raster_line(
            road.a,
            road.b,
            road.width.max(WIDTH as f32 / RESOLUTION as f32 * 0.8),
            |index| {
                colors[index] = if surface.water[index] > 0 {
                    [0.38, 0.32, 0.21]
                } else {
                    [0.39, 0.34, 0.24]
                };
                occupied[index] = true;
            },
        );
    }
    let building_count = props.len();
    let mut random = request.settings.seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        random = random
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (random >> 40) as f32 / 16_777_216.0
    };
    let tree_target = match request.settings.vegetation {
        MapDensity::None => 0,
        MapDensity::Sparse => 100,
        MapDensity::Dense => 260,
    };
    let mut trees = 0;
    let mut rocks = 0;
    for _ in 0..4000 {
        if trees >= tree_target && rocks >= 28 {
            break;
        }
        let position =
            Vec2::new(next() * 2.0 - 1.0, next() * 2.0 - 1.0) * (request.half_extent - 2.0);
        let uv =
            ((position / request.half_extent + Vec2::ONE) * 0.5 * RESOLUTION as f32).as_uvec2();
        let index =
            (uv.y as usize).min(RESOLUTION - 1) * RESOLUTION + (uv.x as usize).min(RESOLUTION - 1);
        if occupied[index]
            || base_positions(request.half_extent)
                .iter()
                .any(|p| p.distance(position) < 5.5)
            || props
                .iter()
                .any(|p| p.position.distance(position) < p.size.x.max(p.size.z) * 0.6 + 0.8)
        {
            continue;
        }
        let kind = if trees < tree_target {
            trees += 1;
            MapPropKind::Tree
        } else {
            rocks += 1;
            MapPropKind::Rock
        };
        let height = 0.9 + next() * 0.8;
        props.push(MapProp {
            kind,
            position,
            size: Vec3::new(0.45 + next() * 0.3, height, 0.45 + next() * 0.3),
            angle: next() * std::f32::consts::TAU,
        });
        if matches!(kind, MapPropKind::Tree) {
            colors[index] = colors[index].map(|c| c * 0.72);
        }
    }
    // Clear visual space around every base while retaining the generated ground elsewhere.
    for y in 0..RESOLUTION {
        for x in 0..RESOLUTION {
            let point = (Vec2::new(x as f32, y as f32) / (RESOLUTION - 1) as f32 * 2.0 - Vec2::ONE)
                * request.half_extent;
            if base_positions(request.half_extent)
                .iter()
                .any(|p| p.distance(point) < 4.5)
            {
                colors[y * RESOLUTION + x] = [0.31, 0.29, 0.22];
            }
        }
    }
    MapData {
        resolution: RESOLUTION,
        half_extent: request.half_extent,
        colors,
        props,
        summary: format!(
            "{settlements} settlements · {building_count} buildings · {trees} trees. Ready to play."
        ),
    }
}
