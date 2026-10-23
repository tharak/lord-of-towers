# Entity catalog

Application entities have `Name` and `EntityKind`. Simulation entities have `MatchEntity` and are replaced on restart. Visual children inherit parent lifetime. The F3 inspector queries live ECS; no duplicate registry or unbounded spawn log is kept.

| EntityKind | Components and relationships | Creation and lifetime |
|---|---|---|
| `Faction` | `Faction { index }` | Five setup records; survive tower loss, replaced on restart. |
| `Tower` | `SimulationId`, `Tower`, `Position`, `Allegiance`, `Health`, `ElementInventory`, `InfluenceEmitter`; Orthanc has `PlayerControlled` | Five setup towers, fixed influence radius 15. NPC towers auto-equip nearby empty slots before FIFO production; player selects five-unit birth recipes. Death drops stock and destroys tower. |
| `Minion` | `SimulationId`, `OriginTower`, `Position`, `Allegiance`, `Health`, `Movement`, `Attack`, `ShootingPause`, `LegState`, `ArmState`, optional `DragControl`, `ElementInventory`, `ElementStats`, `EquippedElements`, `Vision`, `GatheringState`; optional `CombatTarget`, `TowerDefenseTarget`, `TrailEmitter`, private `VisitedTrailMarkers` | Fifteen at setup plus funded births. Empty equipment on birth; stats start at two plus recipe points. Body color fades from faction color toward gray with remaining health percentage. Heals 1 HP/s per permanent Earth point inside a living friendly tower circle, capped at maximum health; overlapping circles do not stack. Death drops cargo and equipment; spent birth recipe is not refunded. Survive origin tower loss. |
| `ElementDeposit` | `ElementDeposit`, `SimulationId`, `Position`, `ElementInventory`; presentation `Stack` | Thirty homogeneous seeded deposits (5–10 units, six per type) plus typed death/reservation drops. Never recharge; empty deposits despawn. |
| `Trail` | `Trail { trip, sequence, remaining }`, `Position`, `Allegiance` | Carrier breadcrumbs only. Pickup and every two traveled units refresh nearest friendly marker within two units or create one. Thirty-second simulation lifetime; independent of carrier death/delivery. Followers mark other visible markers visited on selection and selected marker on arrival. |
| `EquipmentOrder` | `EquipmentOrder { minion, tower, source_position, action }`, `ElementInventory` | At most one per minion. Owns reserved equip unit removed from stock immediately. Completion/cancellation/invalid references despawn order. Cancellation refunds surviving source; destruction drops reservation at recorded source position. Restart clears it. |
| `StackVisual` | Stack child marker, `ChildOf`, primitive mesh/material/transform | One cube per tower health point or torus per deposit unit; reconciled to parent state, removed with parent. |
| `TowerBeacon` | Presentation marker, `ChildOf`, sphere | One above each living tower health stack; stock crown above beacon; removed with parent. |
| `TowerInfluenceVisual` | Presentation marker, `ChildOf`, circle mesh/material | Exact tower influence outline, radius 15 by default. No recharge/gameplay authority. Removed with parent. |
| `ProductionIndicator` | Presentation marker, `MatchEntity`, `ChildOf`, primitive sphere | Player tower only, when at least five unreserved stock units exist. Follows tower height; disappears below threshold or with parent/restart. |
| `DonutVisual` | `DonutVisual`, `ChildOf`, torus/material/transform | One per stock/cargo unit. Towers show five fixed vertical element stacks around the beacon; minion cargo uses eight-piece crown tiers. Element-colored, reconciled to inventory; no inventory authority. Removed with parent. |
| `EquippedElementVisual` | Presentation marker, `MatchEntity`, `ChildOf`, sphere/material/transform | At most three, one per occupied equipment slot in element color. Rebuilt when layout changes; removed with parent/restart. Stores no units. |
| `Projectile` | `Projectile { target, damage, speed, remaining }`, `Position`, `Allegiance`; faction-colored sphere | Free shot; one-second shooter pause. Damage on impact. Despawns on impact, timeout, missing/friendly target or restart. Independent of shooter lifetime. |
| `Battlefield` | Ground primitive mesh/material/transform | Presentation startup, survives restart; samples territory with no ownership authority. |
| `Camera` | Projection/transform, private spectator marker | Startup, survives restart; mouse/touch/keyboard camera controls. |
| `Light` | Lighting/transform | Startup, survives restart. |
| `Ui` | Bevy `Node`, text/button/interaction and private roles | Persistent HUD, toggle, controls and inspector; transient contextual panel described below. |

## UI lifecycle

The five bottom controls are pause/play, 1×, 2×, 4× and restart. `HudRoot`, `HudTitle`, `DesktopDetail`, `ControlBar`, `PauseIcon` and `HudToggleText` are presentation roles. `HudVisibility` survives restart/rotation; its toggle remains accessible when hidden. F3 opens the inspector.

`ContextPanel` is transient UI beside the selected friendly minion or tower. It contains exactly three rows of five element buttons for minions, with stocked choices enabled in each row and equipped choices outlined; towers use one matching row whose fifth selected unit automatically creates a minion and clears selection. Clicking an equipped choice immediately moves the unit into cargo for homeward delivery; pickups fill empty unreserved equipment slots first. Its contextual button/text children are `Ui` entities. Selection/state changes rebuild the panel; deselection, unit destruction or restart removes it. It is independent of persistent HUD visibility. Closing the panel never cancels an accepted tower order. Two fingers pan/pinch; world clicks select units, never draw trails.

## Shared state and rules

`EquippedElements.slots` contains up to three physical units, duplicates allowed. Cargo remains independent. Permanent `ElementStats` starts at two each; a birth consumes five selected/FIFO units and adds matching points only to its newborn. Fire damage, water speed, earth max health and air vision use `base × [1 + .30 × equipped_copies × (1 + .10 × equipped_void_copies)]`. Void alone grants no bonus. Empty gear gives damage 10, speed 4, health 90 and vision 15. Equipment changes preserve health percentage.

`ElementEconomy` tracks initial and consumed counts per type. Deposits + cargo + tower stock + equipped + reserved + consumed equals initial. Visual entities hold no resources. `PlayerCommands` queues validated orders/recipes; `PlayerSelection` stores selected entity, slot and recipe. `TrailIds` allocates carrier trips; per-minion visited memory is pruned as markers disappear.

`CombatTarget` shares movement/shooting target. `TowerDefenseTarget` references a visible attacker and living friendly tower. `PendingTowerAlerts` collects tower shots and resolves recipients inside tower influence with attacker in vision. Defense overrides return orders; otherwise carriers/ordered minions shoot only in-range enemies without pursuing.

`InfluenceField` is bounded 101×101 territory sampling at 10 Hz simulation time. It is not an entity registry and grants no recharge. See [equipment contracts](equipment-contracts.md), [gathering contracts](gathering-contracts.md) and the [wiki](https://github.com/tharak/lord-of-towers/wiki).

Equipment percentages are fixed: Fire +30% base damage, Water +30% base speed, Earth +30% base maximum health, Air +30% base vision, per equipped copy. Void increases these bonuses by 10% per equipped copy. Birth recipe points do not change equipment bonus rates; permanent Earth determines friendly-base healing. One Fire gives 13 damage; Fire + Void gives 13.3; three Fire gives 19. Bases remain 10 damage, 4 units/s, 90 health and 15 vision.

Minions support live mouse/single-touch dragging after eight screen pixels. DragControl holds a clamped ground target and Held/Released phase. Movement follows at upgraded speed, bypassing separation and all autonomous actions until released arrival within one unit. Cargo trails, damage and already-fired projectiles remain active. Pending equipment exchanges resume afterward. LegState is Idle/Moving/DragStart/DragRelease; independent ArmState carrying/shooting flags permit both activities at once. These components prepare for future animations without adding assets.
