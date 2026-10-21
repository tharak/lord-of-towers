# Entity catalog

This catalog describes application entity kinds. The live inspector queries ECS directly to list every current entity ID, name, faction, kind, and component type. Engine-created entities are available through its Engine toggle. Nothing keeps a second entity database or an unbounded spawn log.

Every application-created entity must include `Name` and `EntityKind`. Add a row or update the relevant row whenever a PR introduces entity lifecycles. Inspector entities persist across restarts; all simulation entities carry `MatchEntity` and are replaced on restart.

| Kind / role | Components and relationships | Creation and lifetime | Owning ticket |
|---|---|---|---|
| Faction | `Faction { index }`, `Name`, `EntityKind::Faction`, `MatchEntity` | Five records created during match setup; retain identity even after loss of all towers. Replaced on restart. | #4 |
| Tower | `SimulationId`, `Tower { slot }`, `Position`, `Allegiance(faction entity)`, `Health`, `Spawner`, `RawElement` stock, `ForwardingCursor`, `InfluenceEmitter`, `Name`, `EntityKind::Tower`, `MatchEntity`; presentation adds mesh, material, transform | Five during setup. Capture mutates allegiance/health/timer without destroying tower identity. Restart removes them. | #4, #6, #18 |
| Minion | `SimulationId`, `Minion`, `Position`, `Allegiance(faction entity)`, `OriginTower(tower entity)`, `Health`, `Movement`, `Attack`, `EnergyStore`, `RawElement` cargo, `Vision`, `GatheringState`, optional `TrailEmitter`, `ForwardingCursor`, optional `SupplyConnection`, `InfluenceEmitter`, `Name`, `EntityKind::Minion`, `MatchEntity`; presentation adds mesh, material, transform | Spawned by towers under origin population cap. Death or restart despawns. Capture of origin does not change allegiance. Origin references must be checked before dereferencing. | #4, #5, #6, #18, #19 |
| RawElement deposit | `RawElementDeposit`, `RawElement`, `Position`, optional `SimulationId`, `Name`, `EntityKind::RawElementDeposit`, `MatchEntity`; presentation adds shared torus mesh | 30 seeded deposits with 5–10 units at setup, plus dropped cargo on death. Collection removes one unit; zero stock despawns. No recharge. | #19, #20 |
| Trail marker | `Trail`, `Position`, `Allegiance`, `Name`, `EntityKind::Trail`, `MatchEntity`; presentation adds shared primitive mesh/fade material | Created when collecting and every two units traveled while carrying. Lasts 30 simulation seconds; survives creator death/delivery. Removed on restart. | #19, #20 |
| Donut crown visual | `Name`, `EntityKind::DonutVisual`, `DonutVisual`, `ChildOf`, mesh/material/transform | One torus child per rawElement unit on a minion or tower, in eight-piece tiers. Reconciled after inventory changes; removed with parent, including restart. Carries no inventory or gameplay influence. | #24 |
| Supply link | `Link { source, target }`, `Allegiance`, `Name`, `EntityKind::Link`, `MatchEntity`; presentation adds thin straight mesh/material/transform | Reconciled after combat/capture each fixed step. Unchanged edges retain identity. Removed when endpoints die, move out of range, change faction, or lose tower connectivity. Removed on restart. | #13, #14 |
| Battlefield | `Name`, `EntityKind::Battlefield`, presentation mesh/material/transform | Presentation setup; survives restart. Ground coloring samples influence; terrain carries no gameplay ownership. | #7, #8 |
| Camera | `Name`, `EntityKind::Camera`, camera/projection/transform components | Presentation setup; survives restart. | #8 |
| Light | `Name`, `EntityKind::Light`, lighting/transform components | Presentation setup; survives restart. | #8 |
| Main HUD and controls | `Name`, `EntityKind::Ui`, `Node`, text/button/interaction components as appropriate | Presentation setup; survives restart. State follows resources. | #8 |
| Inspector panel | `Name`, `EntityKind::Ui`, `InspectorRoot`, `Node`, background and z-index | Inspector startup; survives restart; display toggled by `InspectorState.open`. | #3 |
| Inspector control bar | `Name`, `EntityKind::Ui`, `Node`, parent-child relationship | Child of inspector panel, created once at startup. | #3 |
| Inspector buttons and labels | `Name`, `EntityKind::Ui`, `Button` / `InspectorAction` / `Node` or `Text`; parent-child relationship | Created once. Buttons cycle kind/faction filters, unclassified inclusion, entity page, component page, and close. | #3 |
| Inspector detail text | `Name`, `EntityKind::Ui`, `InspectorText`, `Text` and text styles | Created once. Refreshes at most four times per second, one entity and sixteen component type names per page. Despawns automatically disappear on refresh. | #3 |

## Influence lifecycle

`InfluenceField` is a bounded global sampling resource, not an entity registry. Emitters are read from live `Position`, `Allegiance`, and `InfluenceEmitter` components. A 101×101 lattice updates at 10 Hz simulation time and interpolates continuously between samples. Positive finite-radius contributions sum per faction. Within one percent of the strongest contribution, a point is contested; zero influence is neutral. Percentages exclude neutral and contested cells. Restart clears the field immediately. Dead/despawned emitters stop contributing at the next influence update.

## Inspector coverage and bounds

The inspector enumerates live entities, sorting IDs for stable navigation; only transient query results are allocated. Kind and faction filters can be combined. Faction records resolve themselves, while towers/minions resolve `Allegiance`; missing referenced entities are treated as unaffiliated. The optional engine view includes entities without `EntityKind`. Component labels come from registered ECS metadata, using short Rust type names to fit the panel. Next/Previous navigate entities; Components cycles through their component pages. The panel is available through the visible main HUD button or F3.

## Energy lifecycle

Minions own bounded `EnergyStore` components; towers are unlimited energy sources and do not have a finite store. Each participating sender owns its forwarding cursor. `SupplyConnection` exists only on minions currently connected to a friendly tower; its parent/root references are validated against the live world. Every connected minion has exactly one incoming Link, with no cycles or enemy endpoints. There is no persistent second graph registry.

`EnergyClock` is the global simulation-time pulse accumulator. Each pulse snapshots source budgets and recipient capacity before applying transfers. Minions can spend their last point forwarding, but incoming energy cannot be forwarded within the same pulse. The inspector displays energy values, rawElement inventories, vision/gathering state, trail lifetime, and supply/link endpoints alongside component metadata.

## Gathering lifecycle

`RawElement` is a per-entity inventory component on deposits, minions and towers. Minion cargo is bounded to one unit. `RawElementEconomy` records the initial total and cumulative spending, not a second inventory database. Live inventory plus spending equals the initial total. Towers start empty with three free minions; subsequent minions cost five units. Capture retains tower stock, death drops cargo, and no recharge or replacement creates new resources.

`Vision` restricts detection and pursuit to a 15-unit circle; friendly towers are globally known for delivery only. Trail entities carry trip and sequence information, never a hidden resource position. Their expiry and all gathering timers follow simulation pause/speed. Restart removes all deposits, cargo, trails, stocks and runtime navigation state before recreating the seeded initial match.

The [GitHub wiki](https://github.com/tharak/lord-of-towers/wiki) names every application entity and component and distinguishes rawElement, donuts, and torus meshes. Keep this catalog aligned with its Entities page.
