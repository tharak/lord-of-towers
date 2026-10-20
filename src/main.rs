use bevy::prelude::*;
use lord_of_towers::{
    influence::InfluencePlugin, inspector::InspectorPlugin, presentation::PresentationPlugin,
    simulation::SimulationPlugin,
};

fn main() {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(|info| {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            if let Some(error) = document.get_element_by_id("error") {
                error.set_text_content(Some(&format!("The game could not continue: {info}")));
                let _ = error.remove_attribute("hidden");
            }
        }
    }));
    let window = Window {
        title: "Lord of Towers".into(),
        resolution: (1280, 800).into(),
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#game".into()),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        ..default()
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_plugins((
            SimulationPlugin,
            InfluencePlugin,
            InspectorPlugin,
            PresentationPlugin,
        ))
        .run();
}
