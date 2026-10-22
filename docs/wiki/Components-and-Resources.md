# Components and Resources

## Gameplay components

| Contract | Responsibility |
|---|---|
| `Faction`, `Allegiance`, `OriginTower` | Faction record, current owner and birth-tower reference. Survivors keep allegiance after origin loss. |
| `Tower`, `PlayerControlled`, `Minion` | Structure, player faction anchor and mobile unit. |
| `Name`, `EntityKind`, `MatchEntity`, `SimulationId` | Human name, entity catalog category, restart scope and deterministic simulation identity. |
| `Position`, `Movement` | Ground-plane coordinates, speed and navigation target. |
| `Health` | Current/max health; equipment preserves health ratio. Towers destroyed at zero. |
| `Attack`, `ShootingPause` | Damage/range/cooldown and one-second movement pause after a free shot. |
| `Projectile` | Live hostile target, damage, speed and remaining lifetime; impact damage. |
| `CombatTarget`, `TowerDefenseTarget` | Shared firing/pursuit target and persistent alerted tower/attacker pair. |
| `ElementInventory` | FIFO physical units, independent for deposits, minion cargo, tower stock and reservations. |
| `EquippedElements` | Three optional physical element slots; duplicates allowed. |
| `ElementStats` | Permanent fire/water/earth/air/void birth points. Start at two plus consumed recipe counts. |
| `EquipmentOrder` | Recipient, source tower, recorded source position and Equip/ReturnEquipment/DeliverCargo action. Own inventory holds reservation. |
| `ElementDeposit` | Homogeneous neutral finite resource pile; depleted piles despawn. |
| `Vision` | Local detection radius, derived from air/void equipment. |
| `GatheringState`, `TrailEmitter` | Explore/follow/return navigation and active carrier breadcrumb progress. |
| `Trail`, private `VisitedTrailMarkers` | Thirty-second breadcrumb and per-minion visited identities. Selection marks all other visible friendly markers visited; arrival marks selected marker. |
| `InfluenceEmitter` | Fixed radius/strength for territory and tower alert recipients. |

Fire modifies damage, water speed, earth maximum health, air vision. Attribute multiplier is `1 + .10 × matching_stat × equipped_copies × (1 + .10 × void_stat × equipped_void_copies)`. No matching equipped copies gives base values (10/4/30/15). Void alone grants no bonus. Birth gear is empty.

## Visual and UI components

| Contract | Responsibility |
|---|---|
| `Stack`, `StackVisual` | Generic count-derived primitive stacks; health cubes and deposit torus units. |
| `DonutVisual` | Typed cargo/stock crown unit; no inventory authority. |
| `EquippedElementVisual` | Typed sphere for occupied gear slot, up to three; no inventory authority. |
| `TowerBeacon`, `TowerInfluenceVisual`, `ProductionIndicator` | Beacon sphere, radius outline and five-available-unit player production marker. |
| `HudRoot`, `HudTitle`, `DesktopDetail`, `ControlBar`, `PauseIcon`, `HudToggleText` | Responsive hideable HUD, five bottom controls, shape-based pause/play symbols and persistent toggle. |
| `ContextPanel`, contextual action buttons/text | Transient selected minion equipment (three rows of five element buttons) or a single matching tower recipe row, with selected counts. The fifth selected unit automatically queues a birth and clears selection. Equipped buttons are outlined; clicking one moves it immediately into cargo for homeward delivery. Tracks world entity, fits viewport, removed on deselection/death/restart. Closing does not cancel orders. |

All visuals use Bevy primitive meshes and derive from authoritative gameplay state. Child visuals disappear with parents; infrastructure persists across restart.

## Resources and commands

| Contract | Responsibility |
|---|---|
| `SimulationConfig` | World/combat/gathering tuning; delivery radius 3, tower radius 15, carrier spacing 2, trail lifetime 30. |
| `MatchState`, `FactionEntities` | Pause/speed/winner/restart/browser state and faction entity references. |
| `PlayerCommands` | Queued Equip, ReturnEquipment, DeliverCargo, CancelOrder and exact-five CreateMinion requests. Simulation validates ownership/live state/stock. |
| `PlayerSelection` | Selected entity, equipment slot and selected five-unit recipe counts. |
| `ElementEconomy` | Initial and consumed counts per type; deposits + cargo + stock + equipped + reserved + consumed = initial. |
| `TrailIds` | Carrier-trip identity sequence. |
| `PendingTowerAlerts` | Shot-triggered tower defense alerts drained after firing. |
| `CameraGesture`, private pointer state | Two-finger pan/pinch and synthetic mouse suppression. |
| `HudVisibility`, inspector state | Persistent HUD visibility and live inspector controls. |
| `InfluenceField` | Bounded 101×101 territory sampling, 10 Hz simulation updates; no recharge. |

Fixed steps run at 30 Hz, scale with speed and stop when paused/hidden/won. Invalid-order reservation cleanup also runs outside simulation gating. Restart clears match-scoped entities, commands and selection. Equipment transfers happen before NPC production. Player production is explicit; NPCs fill nearby empty slots before consuming five FIFO stock units. Destruction drops cargo/equipment and reservations at recorded source position; cancellation refunds living source stock.

`PlayerInteractionPlugin` replaces all drawing/erasing. Carrier-only trails retain local freshness navigation. Combat prioritizes visible tower defense, then homeward orders/cargo, then visible enemy pursuit, deposits, trails and exploration. Returning units shoot in-range enemies without chasing. No energy component, clock, cost or recharge remains.

New pickups fill the first empty equipment slot not reserved by an exchange. Otherwise the pickup becomes gathered cargo. ReturnEquipment moves gear into cargo immediately; that cargo may accompany one gathered unit, stays carried if an order is replaced, and is credited to stock only on delivery.
