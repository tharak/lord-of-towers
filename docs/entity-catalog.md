# Entity catalog

This catalog describes application entity kinds. The live inspector queries ECS directly to list every current entity ID, name, faction, kind, and component type. Engine-created entities are available through its Engine toggle. Nothing keeps a second entity database or an unbounded spawn log.

Every application-created entity must include `Name` and `EntityKind`. Add a row or update the relevant row whenever a PR introduces entity lifecycles. Inspector entities persist across restarts; all simulation entities carry `MatchEntity` and are replaced on restart.

| Kind / role | Components and relationships | Creation and lifetime | Owning ticket |
|---|---|---|---|
| Faction | `Faction { index }`, `Name`, `EntityKind::Faction`, `MatchEntity` | Five records created during match setup; retain identity even after loss of all towers. Replaced on restart. | #4 |
| Tower | `SimulationId`, `Tower { slot }`, `Position`, `Allegiance(faction entity)`, `Health`, `Spawner`, `InfluenceEmitter`, `Name`, `EntityKind::Tower`, `MatchEntity`; presentation adds mesh, material, transform | Five during setup. Capture mutates allegiance/health/timer without destroying tower identity. Restart removes them. | #4, #6 |
| Minion | `SimulationId`, `Minion`, `Position`, `Allegiance(faction entity)`, `OriginTower(tower entity)`, `Health`, `Movement`, `Attack`, `InfluenceEmitter`, `Name`, `EntityKind::Minion`, `MatchEntity`; presentation adds mesh, material, transform | Spawned by towers under origin population cap. Death or restart despawns. Capture of origin does not change allegiance. Origin references must be checked before dereferencing. | #4, #5, #6 |
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
