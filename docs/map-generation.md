# Map creation and the Magna Urbis boundary

The game automatically prepares the default medieval city (seed 42) and starts the match after generation succeeds. Map options, preview/start prompts, and the map launcher are temporarily hidden. A compact startup message is shown during generation, then disappears when play begins. Restart and R reuse the generated map and seed.

`MapCreation::default` controls this temporary flow through `options_enabled = false`. Re-enable that flag in source to restore the retained map editor: Landscape + villages, Medieval town, and Terrain only presets; independent relief, water, vegetation, settlements, roads/streets, climate, and seed controls; preview generation followed by Start match. The editor supports mouse-wheel and single-touch scrolling and keeps HUD visibility independent.

Automatic generation uses the dense medieval town preset with plains, rivers, streets and sparse trees. If that returns an error, the Magna Urbis adapter retries the same city and seed without water. City generation rejects an empty building result, so it does not silently substitute a terrain-only map. If both attempts fail, startup remains stopped and exposes a Retry map button. The fallback stays entirely inside the adapter; gameplay and rendering continue consuming only our map data.

## Integration boundary

`src/integrations/magna_urbis/` is the only application code allowed to import or call `magna_urbis_core`. Its generation entry points accept our `MapGenerationRequest` and return `Result<MapData, String>`; `generate_default` adds the automatic-start fallback around normal generation. These types are owned by Lord of Towers and contain no upstream types, mesh handles, or material handles. All translation, terrain sampling, hydrology calls, settlement placement, road routing, urban quarters, and parcel generation stay inside the adapter. UI orchestration calls that entry point; gameplay and rendering consume our own data.

- `terrain.rs`: translates relief and water choices into the upstream CPU terrain and hydrology stages.
- `settlements.rs`: translates town settings, retains the five base anchors, and converts upstream roads and parcels into bounded primitive-friendly descriptions.
- `decoration.rs`: produces terrain colors and deterministic vegetation/rock descriptions in game coordinates.

The pinned engine snapshot and original GPLv3 license live in `third_party/magna-urbis/`; see its README and the root third-party notices for revision and provenance. The required house-admission catalogue is included as data; the separate house-generation engine is not used. Do not call the generator directly from a UI system, renderer, entity constructor, or gameplay system. Upgrade the snapshot and adapter together.

## Rendering and gameplay

The adapter samples a 1.2 km × 1.2 km generated region into the existing 100 × 100 game-unit battlefield: one game unit represents 12 metres. Terrain elevation and slopes drive shaded terrain colors; the playable surface remains flat. River/lake and road/street layers are baked into those colors. Parcel footprints become bounded cuboid buildings with primitive roofs; trees and rocks use shared Bevy primitive meshes/materials. Scenery is capped at 400 buildings, 260 trees, and 28 rocks, with clear space around the five base anchors. Influence tint blends over the generated colors so ownership and terrain remain readable together.

Buildings and trees now block minion sight using independent simulation footprints. Rendering entities have no gameplay authority. Terrain remains flat and passable: minions can still cross water, buildings and trees. Base positions, elemental conservation, combat values and healing remain the same. A selected numeric seed also seeds the match.

Map creation gates simulation through `MatchState.map_ready`; pause, hidden tabs, and victory still apply after starting. In automatic mode, a generation error triggers the simpler adapter fallback; if that fails too, startup shows retry controls. In editor mode, a generation error keeps the player in settings. Generation starts after the loading message has had time to render. Native builds use the compute pool; WebAssembly uses Bevy's single-threaded task implementation, so CPU generation can temporarily occupy the browser's main thread. No server, WebGPU requirement, or neural model download is introduced.

`MapLandscape` owns renderer-independent preview data. Its revision independently replaces the `MapScene` root and all `MapDecoration` children, and the `MapVisionRoot` with its `VisionObstacle` children. Scenery survives match restart and is replaced only after another successful generation; settings UI and scene assets survive match cleanup. Runtime behavior has not been smoke-tested: repository validation is native compilation and a release WebAssembly build in GitHub Actions only.


## Minion line of sight

`VisionBlocker` and `Position` define sight footprints without mesh/material handles. Buildings use rotated rectangular wall footprints; tree canopies use rotated ellipses matching their horizontal visual extents. Rocks do not block sight. This is a two-dimensional rule independent of prop or minion height. Units within the same footprint can see one another; a line crossing its boundary is occluded.

The simulation builds `VisionOcclusion`, a four-game-unit spatial grid, from live blocker entities before targeting. It refreshes when blockers are added, moved, changed or removed. The cache survives ordinary match restart with the unchanged map. Installing a new map replaces the independent blocker hierarchy and refreshes its index before minions act. Deleting visual scenery alone does not change visibility.

Range and clear line of sight are both required for enemy acquisition, tower-defense responses, new shots, deposit discovery/pickups, and trail following. Occluded targets are discarded; marker visitation only considers visible markers. Friendly delivery locations remain globally known. Existing projectiles retain their homing behavior, and movement remains passable; this change adds no navigation obstacles or player fog of war. All factions use the same per-minion sight rules.
