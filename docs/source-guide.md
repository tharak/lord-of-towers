# Editing the source

The code follows Bevy ECS: components describe individual entities, resources hold shared state, entity constructors assemble components, and systems change the world. Plugins register resources and order systems. The entry point installs plugins in the same order as before this reorganization.

## Where to make a change

| Change | Start here |
|---|---|
| Health fields or component-specific methods | [Health](../src/components/health.rs) |
| Starting health, speed, tower radius, or world configuration | [SimulationConfig](../src/resources/simulation_config.rs) |
| Components placed on a newborn minion | [Minion construction](../src/entities/minion.rs) |
| Base healing eligibility or rate | [Healing system](../src/systems/simulation/healing.rs) |
| Shooting, projectiles, damage, and death | [Combat systems](../src/systems/simulation/combat.rs) |
| Movement, dragging, or enemy selection | [Simulation systems](../src/systems/simulation/mod.rs) |
| Equipment commands, reservations, or attribute bonuses | [Equipment systems](../src/systems/equipment/mod.rs) |
| Gathering, delivery, exploration, or trails | [Gathering systems](../src/systems/gathering/mod.rs) |
| Pointer selection and contextual actions | [Interaction systems](../src/systems/interaction/mod.rs) |
| Health fading, inventory visuals, camera, or HUD updates | [Presentation systems](../src/systems/presentation/mod.rs) |
| HUD, context-panel, or inspector layout and children | [UI construction](../src/entities/ui/mod.rs) |
| System ordering or resource registration | [Plugins](../src/plugins/mod.rs), especially [simulation registration](../src/plugins/simulation_plugin.rs) |
| Automatic map startup, hidden options, or preview generation | [Map creation systems](../src/systems/map_creation/mod.rs) and [map-generation guide](map-generation.md) |
| Magna Urbis API translation and upgrades | [Adapter boundary](../src/integrations/magna_urbis/mod.rs); all upstream imports and calls stay in this folder |
| Pause conditions and simulation speed scaling | [Scheduling](../src/scheduling/mod.rs) |

For example, to increase base minion life, change `SimulationConfig::default` rather than its constructor or `Health`. To change how life regenerates, edit the healing system. To add data to every minion, define its component in a matching file and add it to the minion constructor. Register any new system in the owning plugin and document entity/lifecycle changes in the entity catalog.

## Folder responsibilities

- `components/`: exactly one custom `Component` definition per snake_case file. This includes small marker components and UI roles. Keep a component's inherent methods with its definition.
- `entities/`: construction functions grouped by entity kind, including visual entities and UI trees. Match entities are rebuilt on restart; children inherit their parent's lifetime. The [entity catalog](entity-catalog.md) describes exceptions and lifecycle details.
- `resources/`: one shared application resource per file, with its default and related methods. Simulation resources never depend on visual assets.
- `systems/`: behavior grouped by subsystem and responsibility. Systems decide when to create, update, or remove entities, then call entity constructors. Browser-specific integration remains isolated behind target configuration.
- `plugins/`: resource initialization and system registration. Ordering constraints belong here so they can be reviewed together.
- `types/`: supporting enums, commands, constants, and ordinary structs. These are not additional ECS components or resources.
- `scheduling/`: simulation sets, run conditions, and time scaling.

Each folder's `mod.rs` is a navigation index; scheduling also contains the small shared schedule contract. New code imports from these folders directly. Keep internal UI and implementation details `pub(crate)`; moving a type does not make it part of the public API.

## Compatibility and validation

`model.rs` re-exports the original shared API, so `model::Health` and similar imports still work. The old subsystem modules likewise preserve public plugin and helper imports. They contain no gameplay implementation; new code should use the dedicated modules.

This reorganization preserves gameplay values, fixed-step ordering, deterministic IDs and random sequences, deferred command timing, parent-child relationships, restart behavior, and native/WebAssembly target conditions. Minions retain 90 base life, faction-color fading, and 1 HP per simulation second per permanent Earth point inside a living friendly tower circle.

Validation remains compilation only: GitHub Actions runs native `cargo check --locked` and a release WebAssembly build. Formatting is allowed. Do not add or run automated tests, browser suites, extended simulations, or runtime smoke checks. Local compilation requires an explicit user request.
