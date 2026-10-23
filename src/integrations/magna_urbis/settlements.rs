//! Upstream settlement, road, quarter, and parcel stages remain private to the adapter.
use super::terrain::{RESOLUTION, Surface, WIDTH};
use crate::types::{
    map_data::{MapProp, MapPropKind},
    map_settings::{MapDensity, MapGenerationRequest, MapPreset, MapRoads},
};
use bevy::prelude::*;
use magna_urbis_core::{
    settlements::{
        self, Point, SettlementSpec,
        roads::{RoadConfig, RoadSite},
    },
    urban,
};
pub(super) struct RoadStroke {
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub width: f32,
}

pub(super) fn base_positions(half_extent: f32) -> [Vec2; 5] {
    std::array::from_fn(|i| {
        Vec2::from_angle(i as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2)
            * 32.0_f32.min(half_extent * 0.75)
    })
}
pub(super) fn generate(
    surface: &Surface,
    request: &MapGenerationRequest,
) -> Result<(Vec<RoadStroke>, Vec<MapProp>, usize), String> {
    let settings = &request.settings;
    let seed = settings.seed.to_string();
    let scale = request.half_extent * 2.0 / WIDTH as f32;
    let to_point = |position: Vec2| Point {
        x: ((position.x + request.half_extent) / scale) as f64,
        y: ((position.y + request.half_extent) / scale) as f64,
    };
    let terrain = settlements::Terrain::new(settlements::Raster {
        width: WIDTH,
        n: RESOLUTION,
        height: &surface.height,
        water: &surface.water,
        water_level: &surface.level,
        river_width: &surface.river_width,
        river_ids: &surface.river_ids,
    })?;
    let mut sites = Vec::new();
    if settings.settlements != MapDensity::None {
        let locations = if settings.preset == MapPreset::Town {
            vec![Vec2::ZERO]
        } else {
            base_positions(request.half_extent).to_vec()
        };
        let config = settlements::Config {
            mode: "list".into(),
            list: locations
                .iter()
                .enumerate()
                .map(|(i, &position)| SettlementSpec {
                    key: format!("town-{i}"),
                    name: format!("Settlement {}", i + 1),
                    population: match (settings.preset, settings.settlements) {
                        (MapPreset::Town, MapDensity::Dense) => 9000,
                        (_, MapDensity::Dense) => 1800,
                        _ => 450,
                    },
                    position: Some(to_point(position)),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        sites = terrain.place(&seed, &config)?.settlements;
    }
    let mut road_sites: Vec<_> = sites
        .iter()
        .map(|site| RoadSite {
            key: site.spec.key.clone(),
            population: site.spec.population,
            culture: site.spec.culture.clone(),
            center: site.center,
            radius: site.radius,
        })
        .collect();
    // The five bases remain the game's anchors, even on terrain-only maps.
    for (i, position) in base_positions(request.half_extent).into_iter().enumerate() {
        let center = to_point(position);
        if road_sites
            .iter()
            .any(|site| (site.center.x - center.x).hypot(site.center.y - center.y) < 1.0)
        {
            continue;
        }
        road_sites.push(RoadSite {
            key: format!("base-{i}"),
            population: 300,
            culture: "medieval".into(),
            center,
            radius: 35.0,
        });
    }
    let mut strokes = Vec::new();
    let mut regional = Vec::new();
    if settings.roads != MapRoads::None {
        let roads = terrain.roads(
            &seed,
            &road_sites,
            &RoadConfig {
                exits: 0,
                ..Default::default()
            },
        )?;
        for segment in roads.segments {
            for pair in segment.path.windows(2) {
                strokes.push(RoadStroke {
                    a: (pair[0].x as f32, pair[0].y as f32),
                    b: (pair[1].x as f32, pair[1].y as f32),
                    width: segment.width.max(12.0) as f32,
                });
            }
            regional.push(urban::Street {
                widths: vec![segment.width; segment.path.len()],
                path: segment.path,
                rank: 0,
                role: "regional".into(),
            });
        }
    }
    if sites.is_empty() {
        return Ok((strokes, Vec::new(), 0));
    }
    let urban_sites: Vec<_> = sites
        .iter()
        .map(|site| urban::Site {
            key: site.spec.key.clone(),
            name: site.spec.name.clone(),
            population: site.spec.population,
            culture: site.spec.culture.clone(),
            center: site.center,
            radius: site.radius,
        })
        .collect();
    let layouts = terrain.urban_quarters(
        &seed,
        &urban_sites,
        &urban::LayoutConfig {
            fortification: "none".into(),
            suburbs: "none".into(),
            ..Default::default()
        },
        &regional,
    )?;
    if settings.roads == MapRoads::Streets {
        for layout in &layouts {
            for street in &layout.streets {
                for pair in street.path.windows(2) {
                    strokes.push(RoadStroke {
                        a: (pair[0].x as f32, pair[0].y as f32),
                        b: (pair[1].x as f32, pair[1].y as f32),
                        width: street.widths.first().copied().unwrap_or(5.0).max(8.0) as f32,
                    });
                }
            }
        }
    }
    let partitions = urban::generate_parcels(&seed, &layouts, &urban::ParcelConfig::default())?;
    let mut buildings = Vec::new();
    let per_site = 400 / partitions.len().max(1);
    for partition in partitions {
        let mut count = 0;
        for parcel in partition.parcels {
            if count >= per_site {
                break;
            }
            if parcel.shape.outer.len() < 3
                || parcel.use_kind.starts_with("open.")
                || parcel.use_kind == "garden"
            {
                continue;
            }
            let vertices: Vec<Vec2> = parcel
                .shape
                .outer
                .iter()
                .map(|p| {
                    Vec2::new(p.x as f32, p.y as f32) * scale - Vec2::splat(request.half_extent)
                })
                .collect();
            let center = vertices.iter().copied().sum::<Vec2>() / vertices.len() as f32;
            if base_positions(request.half_extent)
                .iter()
                .any(|p| p.distance(center) < 5.5)
                || center.abs().max_element() > request.half_extent - 2.0
            {
                continue;
            }
            let axis = Vec2::new(parcel.axis.x as f32, parcel.axis.y as f32)
                .try_normalize()
                .unwrap_or(Vec2::X);
            let side = Vec2::new(-axis.y, axis.x);
            let mut lo = Vec2::splat(f32::INFINITY);
            let mut hi = Vec2::splat(f32::NEG_INFINITY);
            for vertex in &vertices {
                let delta = *vertex - center;
                let q = Vec2::new(delta.dot(axis), delta.dot(side));
                lo = lo.min(q);
                hi = hi.max(q);
            }
            let size = ((hi - lo) * 0.72).clamp(Vec2::splat(0.22), Vec2::splat(2.0));
            buildings.push(MapProp {
                kind: MapPropKind::Building,
                position: center,
                size: Vec3::new(size.x, 0.45 + (count % 4) as f32 * 0.12, size.y),
                angle: -axis.y.atan2(axis.x),
            });
            count += 1;
        }
    }
    Ok((strokes, buildings, sites.len()))
}
