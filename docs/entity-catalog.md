# Entity catalog

Every application-created entity has `Name` and `EntityKind`. Simulation entities also have `MatchEntity` and are replaced on restart. Visual children inherit their parent lifetime. Meshes and materials are presentation state only. The inspector enumerates the live ECS world, including optional engine entities; it stores no second entity registry or unbounded spawn log.

| EntityKind | Components and relationships | Creation and lifetime |
|---|---|---|
| `Faction` | `Faction { index }` | Five records at setup; remain after faction tower loss; replaced on restart. |
| `Tower` | `SimulationId`, `Tower { slot }`, `Position`, `Allegiance`, `Health`, `RawElement`, `InfluenceEmitter`; Orthanc alone has `PlayerControlled` | Five at setup, each with three initial minions. No later minion spawning. Destroyed at zero health, dropping stock. Radius is 15 + retained stock by default. |
| `Minion` | `SimulationId`, `Minion`, `OriginTower`, `Position`, `Allegiance`, `Health`, `Movement`, `Attack`, `EnergyStore`, `ShootingPause`, `RawElement`, `Vision`, `GatheringState`, optional `CombatTarget`, `TowerDefenseTarget`, `TrailEmitter` and private `VisitedTrailMarkers` | Fifteen at setup. Death drops cargo and despawns. Origin tower death does not change allegiance or destroy survivors; origin references are validated. Combat targets are reselected each tick; defense targets end when the attacker leaves vision or either referenced entity becomes invalid. |
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

## UI lifecycle

`HudRoot`, `HudTitle`, `DesktopDetail` and `ControlBar` are private presentation components for responsive HUD sections. The centered bar contains seven controls: pause/play, three speeds, restart, Draw and Erase. `PauseIcon(bool)` identifies rectangle-based UI icon pieces. There is no Entities button; F3 opens the inspector.

The standalone top-right Hide HUD / Show HUD control uses private `HudToggleText`. It remains accessible when HUD content is hidden. `HudVisibility` is a persistent resource whose choice survives match restart and viewport rotation. Inspector root, text, buttons and filters also survive restart; live query refreshes remove stale entities from the display automatically.

## Match-wide state

`RawElementEconomy { initial }` records the initial total; remaining deposits + carried cargo + tower stock always equal it. Tower stock is retained, never spent. `TrailIds` uniquely allocates both carrier and player trips plus player marker serials. Private navigation memory is pruned when individual markers vanish; missing following markers reset navigation so expiry, erasure and eviction cannot strand units.

`CombatTarget` shares the selected hostile entity between movement and shooting. `TowerDefenseTarget` references the alerted tower and attacker, interrupting cargo return only while valid. `PendingTowerAlerts` holds shot-triggered notifications until recipient resolution after firing; the queue is consumed each tick and cleared on restart. Recipients are friendly minions inside the tower circle that can see the attacker.

`EnergyClock` accumulates simulation time for one-energy recharge pulses every five seconds. Friendly tower circles are inclusive, default radius 15 + stock, and overlapping circles grant only once per pulse. Enemy influence and contested terrain do not block recharge. Stores cap at five.

`InfluenceField` is a bounded 101×101 sampling resource updated at 10 Hz simulation time, with interpolated radial contributions and contested/neutral ground. It is not an entity registry and does not decide recharge eligibility. Dead emitters stop contributing on update; restart clears the field.

The wiki [Entities](https://github.com/tharak/lord-of-towers/wiki/Entities) and [Components and Resources](https://github.com/tharak/lord-of-towers/wiki/Components-and-Resources) use the same contracts.
