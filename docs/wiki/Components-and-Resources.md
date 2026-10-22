# Components and Resources

## Gameplay components

| Name | Meaning |
|---|---|
| `Name, EntityKind, MatchEntity` | Application identity, category and restart lifetime. |
| `SimulationId` | Stable match-local tie ordering, distinct from Bevy entity IDs. |
| `Faction, Allegiance` | Faction record and owner reference. |
| `Tower, Minion, RawElementDeposit` | Gameplay roles. Five towers and fifteen initial minions; no later spawning. |
| `PlayerControlled` | Orthanc alone accepts new player drawings; control never transfers after its loss. |
| `OriginTower` | Initial creator reference, validated before use; origin death does not destroy a minion. |
| `Position, Health, Movement` | X/Z ground position, health, steering speed and target. |
| `Attack, ShootingPause` | Shot damage/range/cooldown and one-second movement pause after firing. Shots cost one energy. |
| `CombatTarget` | Enemy selected for shared movement/firing decisions each tick. |
| `TowerDefenseTarget` | Defended tower and firing attacker; valid only while the attacker is visible and both references remain valid. |
| `EnergyProjectile` | Live hostile target, damage, speed and remaining flight lifetime; damage on impact. |
| `RawElement` | Finite deposit quantity, cargo 0–1, or retained tower stock. No visual child holds it. |
| `Vision` | Inclusive local detection circle, default 15. |
| `GatheringState` | Exploring, Following { trip, sequence }, or Returning { tower }. |
| `Trail` | Shared carrier/player trip, path sequence and remaining lifetime, default 30 seconds. Highest remaining lifetime wins navigation selection. |
| `TrailEmitter` | Carrier trip, next sequence and last marker position; emission spacing two. |
| `PlayerDrawnTrail` | Serial provenance for the ten-marker global budget and whole-stroke erase; no separate navigation behavior. |
| `VisitedTrailMarkers` | Private per-minion reached-marker identities, preventing repeat visits; pruned when markers disappear. |
| `EnergyStore` | Combat energy initially five, capacity five; friendly tower circles recharge one per pulse. |
| `InfluenceEmitter` | Finite radial territory contribution. Tower radius defaults to 15 + retained stock. |

## Presentation components

| Name | Meaning |
|---|---|
| `Stack, StackVisual` | Health cubes or deposit torus counts/layout. |
| `TowerBeacon` | Sphere child atop tower health stack. |
| `TowerInfluenceVisual` | Private tower child marking exact faction-colored radius outline; matching EntityKind. |
| `DonutVisual` | One torus crown child per actual rawElement unit. |
| `EnergyOrbVisual` | One sphere child per stored minion energy point. |
| `SpectatorCamera` | Private camera control role. |
| `StatusText, FactionText, Control` | Private HUD labels and clickable actions. |
| `HudRoot, HudTitle, DesktopDetail, ControlBar` | Private responsive HUD roles. |
| `PauseIcon(bool)` | Private rectangular pause/play UI icon pieces. |
| `HudToggleText` | Private label on standalone Hide HUD / Show HUD control. |
| `InspectorRoot, InspectorText, InspectorAction` | Inspector panel, live details and controls. |

Visual components hold no inventory or energy and use Bevy primitive meshes. Child visuals follow parent lifetime; persistent UI survives restart.

## Resources

| Name | Meaning |
|---|---|
| `SimulationConfig` | World, seed, combat, gathering, trail and energy tuning; tower_influence_radius=15, tower_radius_per_raw=1, drawn_trail_capacity=10, energy_recharge_ticks=5. |
| `DrawTrailRequests` | Completed ordered world-space strokes queued for placement. |
| `EraseTrailRequests` | World locations requesting whole player-stroke erasure within two units of marker/segment. |
| `TrailIds` | Shared next_trip for both producers and next_player_marker serial; reset on restart. |
| `DrawingTrail` | Private bounded in-progress pointer samples, touch suppression and tool state. |
| `TrailTool, CameraGesture` | Draw/Erase selection and queued two-finger pan/pinch. |
| `HudVisibility` | HUD choice retained across restart and orientation changes. |
| `MatchState, FactionEntities` | Pause/speed/hidden tab/winner/restart/elapsed state and faction IDs. |
| `RawElementEconomy` | Initial resource total only: remaining + carried + stock = initial. |
| `EnergyClock` | Simulation-time recharge accumulator and completed pulse count. |
| `InfluenceField` | Bounded territory samples and percentages; not recharge authority. |
| `InspectorState, InspectorFilter` | Inspector visibility and filters. |
| `GatheringRuntime, RandomStream, SpawnSequence` | Private seeded randomness and stable identity allocation; no later minion spawning. |
| `PendingTowerAlerts` | Tower/attacker shot records, consumed after firing each tick; alerts friendly minions inside influence that can see the attacker. Cleared on restart. |
| `SpatialIndex, PendingDamage` | Nearby combat lookup and simultaneous projectile impact resolution. |
| `VisualAssets` | Private shared primitive meshes/materials; no gameplay state. |

`TrailInputPlugin` in `orders.rs` handles drawings; `GatheringPlugin` handles common trail navigation and decay. Raw samples are bounded; accepted points are at least half vision radius from every accepted point of the same stroke, with no forced endpoint. Separate strokes may overlap. Carriers retain two-unit spacing.

Bevy `Time<Fixed>` supplies 30 Hz steps. Simulation timers scale with speed and stop while paused, hidden or won. Area recharge grants at most one energy per minion per five-second pulse inside any friendly inclusive tower circle, even in contested ground.

Movement priorities are tower defense, cargo return, closest visible enemy, visible deposit, freshest unvisited friendly trail, then exploration. Carriers may shoot enemies already in range but chase only for valid tower defense. Insufficient energy skips combat pursuit. Defense ends when the attacker leaves vision or the defended tower/attacker becomes invalid.
