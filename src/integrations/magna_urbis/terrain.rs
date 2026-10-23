//! Converts our terrain options into upstream calls; retains no rendering handles.
use crate::types::map_settings::{MapRelief, MapSettings, MapWater};
use magna_urbis_core::{HydrologyConfig, HydrologyEngine, TerrainGenerator};
pub(super) const WIDTH: f64 = 1200.0;
pub(super) const RESOLUTION: usize = 96;
pub(super) struct Surface {
    pub height: Vec<f32>,
    pub water: Vec<u8>,
    pub level: Vec<f32>,
    pub river_width: Vec<f32>,
    pub river_ids: Vec<i32>,
}
pub(super) fn generate(settings: &MapSettings) -> Result<Surface, String> {
    let seed = settings.seed.to_string();
    let relief = match settings.relief {
        MapRelief::Plains => "flat",
        MapRelief::Hills => "hills",
        MapRelief::Valley => "valley",
        MapRelief::Mountains => "mountains",
    };
    let generator = TerrainGenerator::new(&seed, WIDTH, relief, 0.25, WIDTH)?;
    let terrain = generator.sample_region(0.0, 0.0, WIDTH, RESOLUTION)?;
    let len = RESOLUTION * RESOLUTION;
    let mut surface = Surface {
        level: terrain.height.clone(),
        height: terrain.height,
        water: vec![0; len],
        river_width: vec![0.0; len],
        river_ids: vec![-1; len],
    };
    if settings.water == MapWater::None {
        return Ok(surface);
    }
    let rivers = matches!(settings.water, MapWater::Rivers | MapWater::RiversAndLakes);
    let lakes = matches!(settings.water, MapWater::Lakes | MapWater::RiversAndLakes);
    let config = HydrologyConfig {
        main: if rivers { "river" } else { "none" }.into(),
        density: if rivers { 0.7 } else { 0.0 },
        lakes: if lakes { "some" } else { "none" }.into(),
        min_lake_area: 8000.0,
        lake_abundance: 1.0,
        lake_coverage: 0.12,
        ..Default::default()
    };
    let water = HydrologyEngine::new(&seed, WIDTH, RESOLUTION, &surface.height, false)?
        .generate(&config)?;
    for (index, depth) in water.water_field.iter().enumerate() {
        if *depth > 0.0 {
            surface.water[index] = 2;
            surface.level[index] += depth;
        }
    }
    // Rasterize upstream river paths into the data consumed by placement and our renderer.
    for (river, offsets) in water.river_offsets.windows(2).enumerate() {
        let points = &water.river_points[offsets[0] as usize * 4..offsets[1] as usize * 4];
        let points: Vec<_> = points.chunks_exact(4).collect();
        for segment in points.windows(2) {
            let (a, b) = (segment[0], segment[1]);
            let width = ((a[2] + b[2]) * 0.5).max(WIDTH as f32 / RESOLUTION as f32 * 0.7);
            raster_line((a[0], a[1]), (b[0], b[1]), width, |index| {
                surface.water[index] = 3;
                surface.river_width[index] = width;
                surface.river_ids[index] = river as i32;
                surface.level[index] = (a[3] + b[3]) * 0.5;
            });
        }
    }
    Ok(surface)
}
pub(super) fn raster_line(a: (f32, f32), b: (f32, f32), width: f32, mut visit: impl FnMut(usize)) {
    let cell = WIDTH as f32 / RESOLUTION as f32;
    let min_x = ((a.0.min(b.0) - width) / cell).floor().max(0.0) as usize;
    let max_x = ((a.0.max(b.0) + width) / cell)
        .ceil()
        .min((RESOLUTION - 1) as f32) as usize;
    let min_y = ((a.1.min(b.1) - width) / cell).floor().max(0.0) as usize;
    let max_y = ((a.1.max(b.1) + width) / cell)
        .ceil()
        .min((RESOLUTION - 1) as f32) as usize;
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let length = dx * dx + dy * dy;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = (x as f32 + 0.5) * cell;
            let py = (y as f32 + 0.5) * cell;
            let t = if length > 0.0 {
                ((px - a.0) * dx + (py - a.1) * dy) / length
            } else {
                0.0
            }
            .clamp(0.0, 1.0);
            if (px - a.0 - t * dx).hypot(py - a.1 - t * dy) <= width * 0.5 {
                visit(y * RESOLUTION + x);
            }
        }
    }
}
