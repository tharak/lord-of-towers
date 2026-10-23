# Map creation and the Magna Urbis boundary

The game opens in map creation. Choose Landscape + villages, Medieval town, or Terrain only, then cycle relief, water, vegetation, settlement density, roads/streets, and climate independently. The preset sets starting choices; individual choices can override it. Seed controls decrement, increment, or shuffle a reproducible numeric seed. Generate preview builds the map, and Start match starts the five-faction simulation only after a successful preview. Changing any setting disables Start until another preview is generated.

Show preview hides the settings panel; Map settings brings it back. The panel supports mouse-wheel and single-touch scrolling. During play, New map ends the current match and returns to creation. Restart and R restart the match on the same generated map and seed. HUD visibility remains independent of map creation.

## Integration boundary

`src/integrations/magna_urbis/` is the only application code allowed to import or call `magna_urbis_core`. Its single entry point accepts our `MapGenerationRequest` and returns `Result<MapData, String>`. These types are owned by Lord of Towers and contain no upstream types, mesh handles, or material handles. All translation, terrain sampling, hydrology calls, settlement placement, road routing, urban quarters, and parcel generation stay inside the adapter. UI orchestration calls that entry point; gameplay and rendering consume our own data.

- `terrain.rs`: translates relief and water choices into the upstream CPU terrain and hydrology stages.
- `settlements.rs`: translates town settings, retains the five base anchors, and converts upstream roads and parcels into bounded primitive-friendly descriptions.
- `decoration.rs`: produces terrain colors and deterministic vegetation/rock descriptions in game coordinates.

The pinned engine snapshot and original GPLv3 license live in `third_party/magna-urbis/`; see its README and the root third-party notices for revision and provenance. The required house-admission catalogue is included as data; the separate house-generation engine is not used. Do not call the generator directly from a UI system, renderer, entity constructor, or gameplay system. Upgrade the snapshot and adapter together.

## Rendering and gameplay

The adapter samples a 2.4 km generated region into the existing battlefield bounds. Terrain elevation and slopes drive shaded terrain colors; the playable surface remains flat. River/lake and road/street layers are baked into those colors. Parcel footprints become bounded cuboid buildings with primitive roofs; trees and rocks use shared Bevy primitive meshes/materials. Scenery is capped at 400 buildings, 260 trees, and 28 rocks, with clear space around the five base anchors. Influence tint blends over the generated colors so ownership and terrain remain readable together.

This is a visual map upgrade. Scenery has no collision, pathfinding, combat, gathering, vision, or healing authority: minions can cross water and buildings. Base positions, elemental conservation, combat values, and gathering rules remain the same. A selected numeric seed also seeds the match.

Map creation gates simulation through `MatchState.map_ready`; pause, hidden tabs, and victory still apply after starting. Failed generation keeps the player in settings with an error and retry controls. Generation starts after the loading message has had time to render. Native builds use the compute pool; WebAssembly uses Bevy's single-threaded task implementation, so CPU generation can temporarily occupy the browser's main thread. No server, WebGPU requirement, or neural model download is introduced.

`MapLandscape` owns renderer-independent preview data. Its revision replaces the `MapScene` root and all `MapDecoration` children. Scenery survives match restart and is replaced only after another successful generation; settings UI and scene assets survive match cleanup. Runtime behavior has not been smoke-tested: repository validation is native compilation and a release WebAssembly build in GitHub Actions only.
