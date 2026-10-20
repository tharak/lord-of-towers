# Shared implementation contracts

`model.rs` is owned by the lead. SimulationPlugin owns match setup, restart, fixed schedule and units. InfluencePlugin owns InfluenceField and sampling. PresentationPlugin owns visuals and spectator controls. InspectorPlugin owns the live entity inspector.

Simulation uses Position(Vec2) on X/Z; Y is visual height. Faction index is resolved through Allegiance's faction entity. EntityKind + Name classify application entities; MatchEntity marks everything recreated on restart.

InfluenceField must expose `sample(position: Vec2) -> [f32; 5]`, `territory_percentages() -> [f32; 5]`, public `revision: u64`, and `half_extent: f32`. Influence sampling runs in SimSet::Influence, at 10 Hz simulation time. Presence uses a finite radial falloff, with strongest influence controlling territory and ties contested. Presentation reads InfluenceField to color ground; AI may query sample for target scoring.

SimulationPlugin configures ordered SimSet chain Spawn → Index → Target → Movement → Combat → Capture → Influence → Victory, gated by simulation_running. Fixed time runs at 30 Hz. simulation_delta applies speed. Restart is serviced in Update even while paused. MatchState is the control interface.

Use a 100×100 world, five towers on a radius-32 circle. Initial balance defaults live in SimulationConfig. Independent seeded random choices break symmetry without nondeterministic test expectations.

Presentation builds a shared primitive mesh/material palette and adds visuals to new Tower/Minion entities. Capture recolors from Allegiance. Camera and UI survive match restart. No gameplay logic depends on rendered dimensions. Controls: Space pause, 1/2/3 speed 1×/2×/4×, R restart, F3 inspector; clickable controls are also required. Browser visibility sets browser_hidden, not paused, to preserve user intent.

Inspector exposes a public `InspectorState { pub open: bool }` resource for presentation's button. Avoid browser-blocked F3 as the only entrypoint.
