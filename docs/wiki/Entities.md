# Entities

Every application entity has `Name` and `EntityKind`; match entities carry `MatchEntity` and are replaced on restart. Presentation children follow parent lifetimes. The inspector queries the live world directly.

| EntityKind | Components and relationships | Creation and lifetime |
|---|---|---|
| `Faction` | `Faction { index }` | Five records at setup; remain after faction tower loss; replaced on restart. |
| `Tower` | `SimulationId`, `Tower { slot }`, `Position`, `Allegiance`, `Health`, `RawElement`, `InfluenceEmitter`; Orthanc alone has `PlayerControlled` | Five at setup, each with three initial minions. No later minion spawning. Destroyed at zero health, dropping stock. Radius is 15 + retained stock by default. |
| `Minion` | `SimulationId`, `Minion`, `OriginTower`, `Position`, `Allegiance`, `Health`, `Movement`, `Attack`, `EnergyStore`, `ShootingPause`, `RawElement`, `Vision`, `GatheringState`, optional `TrailEmitter` and private `VisitedTrails` | Fifteen at setup. Death drops cargo and despawns. Origin tower death does not change allegiance or destroy survivors; origin references are validated. |
| `RawElementDeposit` | `RawElementDeposit`, `Position`, `RawElement`, `SimulationId`; presentation `Stack` | Thirty seeded 5–10-unit deposits plus death drops. Collection reduces quantity; empty deposits despawn. Never recharge. |
| `Trail` | `Trail { trip, sequence, remaining }`, `Position`, `Allegiance`; optional `PlayerDrawnTrail { serial }` | Shared carrier/player markers, lasting 30 simulation seconds. Carriers emit every two traveled units. Player points are half a vision radius apart within each stroke; ten live player markers globally, replacing oldest serials. Erasure within two units of a marker/segment removes the whole player trip, never carriers. Survive origin death/delivery; restart removes all. |
| `StackVisual` | Private stack-child marker, `ChildOf`, primitive mesh/material/transform | One cube per tower health point or torus per deposit unit. Reconciled with parent count; removed with parent. |
| `TowerBeacon` | Presentation marker, `ChildOf`, sphere mesh/material/transform | One atop each living tower health stack; crown above it. Removed with parent. |
| `TowerInfluenceVisual` | Private `TowerInfluenceVisual`, `ChildOf`, primitive mesh/material/transform | One faction-colored circular outline per tower at its exact influence radius. Resized after stock growth; removed with parent. No gameplay authority. |
| `DonutVisual` | `DonutVisual`, `ChildOf`, shared torus mesh/material/transform | One crown child per actual tower stock or minion cargo unit, in eight-piece tiers. Reconciled to inventory; removed with parent. Stores no inventory. |
| `EnergyOrbVisual` | Presentation marker, `ChildOf`, shared sphere mesh/material/transform | One orbiting sphere per stored minion energy. Reconciled to store; removed with parent. Stores no energy. |
| `EnergyProjectile` | `EnergyProjectile { target, damage, speed, remaining }`, `Position`, `Allegiance`; sphere presentation | Created on firing after energy cost. Moves independently of shooter; removed on impact, timeout, missing/friendly target or restart. Damage occurs on impact. |
| `Battlefield` | Ground primitive mesh/material/transform | Presentation startup; survives restart. Ground samples territory, holding no gameplay ownership. |
| `Camera` | Camera/projection/transform, private `SpectatorCamera` | Presentation startup; survives restart; responds to keyboard and touch pan/zoom. |
| `Light` | Lighting/transform | Presentation startup; survives restart. |
| `Ui` | Bevy `Node`, text/button/interaction and private role components | HUD, toggle, controls and inspector created at startup; survive restart. Details below. |

HUD and inspector survive restart. The independent Hide HUD / Show HUD toggle retains its setting through restart and rotation. The centered seven controls are pause/play, three speeds, restart, Draw and Erase. F3 opens the inspector; there is no Entities button. See [Components and Resources](Components-and-Resources) for UI roles and shared state.
