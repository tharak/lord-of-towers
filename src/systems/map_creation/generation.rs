//! UI orchestration depends on our adapter entry point, never on upstream APIs.
use crate::{
    integrations::magna_urbis,
    resources::{MapCreation, MapGeneration, MapLandscape, MatchState, SimulationConfig},
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
    mut config: ResMut<SimulationConfig>,
    mut state: ResMut<MatchState>,
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
        let automatic = !map.options_enabled;
        generation.0 = Some(AsyncComputeTaskPool::get().spawn(async move {
            if automatic {
                magna_urbis::generate_default(request)
            } else {
                magna_urbis::generate(request)
            }
        }));
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
            if map.options_enabled {
                map.phase = MapPhase::Ready;
            } else {
                config.seed = map.settings.seed;
                map.phase = MapPhase::Playing;
                state.map_ready = true;
                state.restart_requested = true;
            }
        }
        Err(error) => {
            warn!("Map generation failed: {error}");
            map.message = if map.options_enabled {
                format!(
                    "Could not generate this map: {error}. Change the seed or settings and try again."
                )
            } else {
                "The battlefield could not be prepared. Retry to try again.".into()
            };
            map.phase = MapPhase::Configuring;
        }
    }
}
