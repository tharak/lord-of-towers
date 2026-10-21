# Entities

Every application-created entity has `Name` and `EntityKind`. Gameplay entities carry `MatchEntity` and are replaced on restart. Child visuals inherit their parent lifetime. The live inspector shows their current IDs and components.

| Entity kind | Purpose and main components | Creation and removal |
|---|---|---|
| `Faction` | Faction identity and index. | Five per match; replaced on restart. |
| `Tower` | Capturable root with `Allegiance`, `Position`, `Health`, `Spawner`, `RawElement` stock, `ForwardingCursor`, `InfluenceEmitter`. | Five per match. Capture preserves entity and stock. |
| `Minion` | Mobile unit with `Allegiance`, `OriginTower`, `Position`, `Health`, `Movement`, `Attack`, `EnergyStore`, `RawElement` cargo, `Vision`, `GatheringState`, optional `TrailEmitter` and `SupplyConnection`. | Three start at each tower; paid spawns cost five rawElement. Death drops cargo and despawns the unit. |
| `RawElementDeposit` | Neutral collectable donut with `Position` and `RawElement` quantity. | Thirty seeded deposits plus cargo drops; removed at zero quantity; no recharge. |
| `DonutVisual` | One torus crown piece per rawElement on its parent tower or minion. Holds no inventory. | Visual child added/removed as parent inventory changes; removed with parent. |
| `EnergyOrbVisual` | One small sphere per point in its parent minion's `EnergyStore`. Holds no energy itself. | Visual child added/removed as stored energy changes; removed with parent. |
| `EnergyProjectile` | Fired energy sphere with `Position`, `Allegiance`, target, damage, speed and remaining lifetime. | Spawned by an attack; follows a live hostile target; removed on impact, lost target, timeout or restart. |
| `Trail` | Faction-colored breadcrumb with `Position`, `Allegiance`, trip/sequence and remaining lifetime. | Left every two traveled units by a carrier; expires after 30 simulation seconds. |
| `Link` | Faction-colored supply edge with source and target entities. | Reconciled each fixed step; removed when endpoints or ownership change. |
| `Battlefield` | Ground mesh sampled from influence. | Presentation startup; survives match restart. |
| `Camera` | Spectator camera. | Presentation startup; survives match restart. |
| `Light` | Scene lighting. | Presentation startup; survives match restart. |
| `Ui` | HUD, controls, inspector panel, buttons and text. | Presentation/inspector startup; survives match restart. |

All meshes and materials are presentation state. `DonutVisual` is never a collectable deposit, and `EnergyOrbVisual` never stores energy. Exact component definitions are listed in [Components and Resources](Components-and-Resources).
