//! UI orchestration depends on our adapter entry point, never on upstream APIs.
use crate::{
    integrations::magna_urbis,
    resources::{MapCreation, MapGeneration, MapLandscape, SimulationConfig},
    types::{map_settings::MapGenerationRequest, map_ui::MapPhase},
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, block_on, poll_once},
};
pub(crate) fn generate_preview(
    mut map: ResMut<MapCreation>,
    mut generation: ResMut<MapGeneration>,
    mut landscape: ResMut<MapLandscape>,
    config: Res<SimulationConfig>,
) {
    if map.phase != MapPhase::Generating {
        return;
    }
    if generation.0.is_none() {
        // Let the loading state reach the screen before starting CPU work, including on WASM.
        if map.pending_frames < 2 {
            map.pending_frames += 1;
            return;
        }
        let request = MapGenerationRequest {
            settings: map.settings,
            half_extent: config.half_extent,
        };
        generation.0 =
            Some(AsyncComputeTaskPool::get().spawn(async move { magna_urbis::generate(request) }));
    }
    let Some(task) = generation.0.as_mut() else {
        return;
    };
    let Some(result) = block_on(poll_once(task)) else {
        return;
    };
    generation.0 = None;
    match result {
        Ok(data) => {
            map.message = data.summary.clone();
            landscape.data = Some(data);
            landscape.revision = landscape.revision.wrapping_add(1);
            map.phase = MapPhase::Ready;
        }
        Err(error) => {
            map.message = format!(
                "Could not generate this map: {error}. Change the seed or settings and try again."
            );
            map.phase = MapPhase::Configuring;
        }
    }
}
