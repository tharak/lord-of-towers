# Components and Resources

## Gameplay components

| Component | Meaning |
|---|---|
| `Name`, `EntityKind`, `MatchEntity` | Application label, inspectable category, and match restart lifetime. |
| `SimulationId` | Stable match-local ordering for deterministic ties; distinct from Bevy entity ID. |
| `Faction`, `Allegiance` | Faction record and owner reference. |
| `Tower`, `Minion`, `RawElementDeposit` | Gameplay role markers. Towers are destroyed at zero health. |
| `PlayerControlled` | Marks exactly one tower, Orthanc (slot 0), as the source of player attack orders. |
| `OriginTower` | Tower that spawned a minion; population cap counts living minions by this reference. |
| `Position`, `Health`, `Movement` | World position, health, and steering target/speed. |
| `Spawner` | Population cap for minions from a tower; rawElement payment is handled by the spawn system. |
| `Attack` | Damage, reach, cooldown, and time until the next shot. Eligible enemy minions take priority over towers. Firing spends one energy; damage occurs on projectile impact. |
| `AttackTrail`, `AttackOrder` | Numbered player-drawn path markers with issuing tower, path ID and optional target at the end; a follower's `AttackOrder` references its current marker. Drawing itself assigns no minion. |
| `ShootingPause` | Remaining simulation time that a minion stays still after firing. |
| `EnergyProjectile` | Target, damage, flight speed, and remaining simulation lifetime of a fired energy sphere. |
| `Vision` | Local circular radius; default 15. |
| `RawElement` | Quantity on a deposit, cargo on a minion (0–1), or stock in a tower. No crown child has this component. |
| `GatheringState` | Exploring, following a local trail, or returning to a friendly tower. |
| `TrailEmitter` | Carrier's trip, next marker sequence, and last marker position. |
| `Trail` | Faction breadcrumb trip, sequence and remaining simulation time. |
| `EnergyStore` | Minion combat energy, initially five and capped at five. |
| `ForwardingCursor`, `SupplyConnection`, `Link` | Fair transfer rotation, tower-rooted parent/root, and live supply edge. |
| `InfluenceEmitter` | Finite-radius territory contribution. |

## Presentation and inspector components

| Component | Meaning |
|---|---|
| `DonutVisual` | Presentation-only crown child. One per actual rawElement unit; rendered with shared Bevy `Torus` mesh. |
| `Stack`, `StackVisual` | Reusable visual count/layout and its child primitive; used for tower health cubes and deposit donuts. |
| `TowerBeacon` | Sphere child atop a tower stack; the attack laser starts here. |
| `EnergyOrbVisual` | Presentation-only small sphere child. One per actual stored minion energy point. |
| `SpectatorCamera` | Camera control target. |
| `StatusText`, `FactionText`, `Control` | Debug HUD labels and clickable controls. |
| `InspectorRoot`, `InspectorText`, `InspectorAction` | Inspector panel, details, and navigation/filter controls. |
| Bevy mesh, material, transform, hierarchy and visibility components | Shared primitive rendering, parent-child lifetime, and display state. |

`VisitedTrails` is private per-minion navigation memory used to avoid following an exhausted trip again.

## Match resources

| Resource | Meaning |
|---|---|
| `SimulationConfig` | Default world size, seed, vision, gathering, combat, spawning, trail and energy tuning. |
| `AttackOrderRequests` | Pending screen-to-world mouse/touch paths consumed by the next simulation step. |
| `CancelAttackOrderRequests` | Pending right-click ground points used to remove the whole player path when a marker is within two world units. |
| `MatchState` | Pause, speed, hidden-tab, elapsed time, winner and restart request. |
| `FactionEntities` | Five live faction entity IDs for deterministic ownership lookup. |
| `RawElementEconomy` | Initial resource total and cumulative spending; verifies that live deposits, cargo and stock plus spending equal the initial total. |
| `EnergyClock` | Supply pulse accumulator/count. |
| `InfluenceField` | Bounded influence samples and territory percentages. |
| `InspectorState`, `InspectorFilter` | Inspector visibility and filters. |
| `GatheringRuntime`, `RandomStream`, `SpawnSequence` | Seeded exploration and stable spawn/trip ordering. |
| `SpatialIndex`, `PendingDamage` | Per-step nearby combat lookup and simultaneous resolution of projectile impacts. |
| `VisualAssets` | Shared primitive meshes and materials; no gameplay inventory. |

Bevy `Time<Fixed>` supplies the 30 Hz step. Gameplay timers use simulation speed and stop when paused, hidden, or won.
