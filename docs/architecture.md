# Shared implementation contracts

`src/model.rs` defines shared gameplay contracts and is coordinated by the lead. Gameplay and UI use Bevy ECS. `Position(Vec2)` uses X/Z world coordinates; Y is visual height. `Allegiance` references a faction entity. Every application entity has `Name` and `EntityKind`; `MatchEntity` marks match-owned entities. Simulation never depends on mesh or material handles. Presentation uses only Bevy primitive meshes.

## Plugins and schedule

`SimulationPlugin` owns setup, restart, movement, combat, tower growth and victory. Its CombatTarget step overrides base gathering movement with visible enemy pursuit or tower defense; shooting uses the same selected target. `GatheringPlugin` owns deposits, local navigation, collection, delivery and fading trails. `TrailInputPlugin` in `orders.rs` owns pointer gestures and player trail placement/erasure. `EnergyPlugin` owns area recharge. `InfluencePlugin` owns bounded territory sampling. `PresentationPlugin` owns visuals, camera and HUD; `InspectorPlugin` queries the live world directly.

The ordered fixed sets are Spawn → Index → Target → CombatTarget → TrailInput → Movement → Combat → Capture → Gathering → TowerGrowth → Energy → Influence → Victory. Spawn advances the clock; it does not create more minions. Fixed time runs at 30 Hz; `simulation_delta` applies speed. Pause, hidden tabs and victory gate simulation. Restart runs in PreUpdate even while paused and flushes match despawns before presentation. Five towers and three minions per tower are recreated with the configured seed; no later spawning occurs.

Shots at a tower enqueue `TowerAlert` records in `PendingTowerAlerts`. After firing, alerts select friendly recipients within the attacked tower's inclusive influence radius that can see the attacker. A `TowerDefenseTarget` survives while the attacker remains visible and the defended tower is alive and friendly. Competing alerts choose the closest attacker with stable ID ties. Alerts are consumed in the same tick and cleared on restart; recipients act in the next targeting step.

Tower radius is `tower_influence_radius + tower_radius_per_raw * stock` (defaults 15 and 1). Gathering precedes growth, and energy uses the resulting radius. Stock is retained until tower destruction. See [gathering](gathering-contracts.md) and [energy](energy-contracts.md).

`InfluenceField` exposes `sample(Vec2) -> [f32; 5]`, `territory_percentages() -> [f32; 5]`, `revision` and `half_extent`. Its bounded 101×101 lattice updates at 10 Hz simulation time, with continuous interpolation. Finite radial contributions sum by faction; values within one percent of the strongest are contested. Zero is neutral; percentages exclude contested and neutral cells. Recharge uses exact tower circles rather than sampled territory.

## Presentation and controls

Shared primitive assets render units, health stacks, resource crowns, projectiles, tower radius outlines and both trail sources. Child visuals follow their parent lifetime. Camera, HUD and inspector survive match restart. The [entity catalog](entity-catalog.md) defines all application lifecycles.

The centered control bar has seven buttons: pause/play, 1×, 2×, 4×, restart, Draw, Erase. `PauseIcon(bool)` identifies rectangular UI icon pieces. The independent top-right HUD toggle uses `HudToggleText`; `HudVisibility` retains the user's choice across restart and responsive layout changes. `HudRoot`, `HudTitle`, `DesktopDetail` and `ControlBar` identify private responsive presentation roles. Hiding the HUD leaves its toggle accessible. F3 controls the inspector; no Entities button is present.

`TrailTool` selects Draw or Erase. `CameraGesture` carries pan and pinch scale to presentation. Two fingers discard an unfinished stroke and suppress drawing until all fingers lift. Touch also suppresses synthetic mouse input. UI interaction and an open inspector block world editing. Bounded private `DrawingTrail` samples become `DrawTrailRequests` on release; `EraseTrailRequests` remain responsive while paused. Browser visibility changes `browser_hidden`, preserving the user's pause choice.

## Collaboration and checks

Coordinate shared model changes with the lead. Each agent edits assigned files and commits on its assigned branch; never merge another agent's branch. Record new entity kinds and lifecycles in the catalog and wiki. Validation is compilation only: native `cargo check --locked` and a release WebAssembly build in GitHub Actions. Do not add automated tests or runtime smoke checks. Investigate runtime issues when the user reports them; local compilation runs only when requested.
