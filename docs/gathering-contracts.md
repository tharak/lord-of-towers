# Elements, minion stats and trail contracts

`GatheringPlugin` owns finite deposits, local navigation, collection/delivery and trail decay/emission. `SimulationPlugin` owns element-funded production. Gameplay components live in `model.rs`; meshes and materials belong to presentation.

## Typed inventory and reset

`Element` is Fire (red), Water (blue), Earth (yellow), Air (green), or Void (black), in that stable order. `ElementInventory { units: VecDeque<Element> }` is the sole inventory source: homogeneous deposit units, minion cargo (zero or one), or FIFO tower stock. Counts are derived from this queue. Initial setup creates thirty seeded random deposits, six of each type, with 5–10 units each, five empty towers and three minions per tower.

Pickup moves one typed unit to an empty minion. Delivery appends cargo to friendly tower stock in stable minion-ID order and clears the carrier emitter. Empty deposits despawn and never recharge. Death drops preserve inventory types: a mixed tower inventory becomes one homogeneous deposit per nonempty element type, each with a unique stable drop ID. Spent birth elements are not refunded on death.

`ElementEconomy { initial: [u32; 5], consumed: [u32; 5] }` tracks type totals. For each element, **deposits + cargo + tower stock + consumed = initial**. Spawn setup resets economy totals, gathering randomness, `TrailIds` and request queues. Restart removes all match-owned entities before seeded recreation; visual children never own inventory.

## Production and elemental stats

After gathering/delivery and before recharge, each living tower consumes the oldest five units while a complete recipe is available. Any mixture is valid. Stable tower IDs order births; partial recipes remain stored. Every birth creates a minion near that tower, with its faction, `OriginTower`, a unique `SimulationId`, empty cargo and starting energy.

`ElementStats { fire, water, earth, air, void }` begins at two in every field. Each consumed recipe unit adds one point to the newborn's matching field; existing minions are unchanged. Three fire and two water produces (5, 4, 2, 2, 2).

Fire modifies attack damage, water movement speed, earth maximum health, and air vision. Each uses **base × (1 + 0.10 × matching_stat × (1 + 0.10 × void_stat))**. Base damage is 10, speed 4, health 30, vision 15. Starting points count: initial minions have 12.4 damage, 4.96 speed, 37.2 health and 18.6 vision. Newborn health starts full. Void multiplies earned bonuses, not base values. Tower influence/recharge radius is fixed at the configured default 15; delivery and births never change it.

## Local navigation

Movement priority is valid tower defense alert → cargo return → closest visible enemy → visible deposit → freshest visible friendly trail → seeded exploration. Combat steering follows base gathering navigation. Each minion uses its own stat-derived inclusive vision circle for acquisition and continued pursuit. Friendly delivery tower positions are globally known; there is no shared faction vision. Tower alerts require recipients to see the firing attacker. Insufficient attack energy skips combat pursuit.

Collectors retain a friendly delivery tower until missing or hostile. With no friendly tower they explore while keeping cargo. Pickup radius is one; delivery radius three. Exploration retargets on arrival or every three simulation seconds within map bounds.

`GatheringState` is Exploring, Following { trip, sequence }, or Returning { tower }. Trail selection reevaluates every tick, ranking visible friendly unvisited markers by highest remaining lifetime, then distance and stable identity. Arrival within one unit records that marker in private `VisitedTrailMarkers` and immediately selects another candidate. Missing eligible markers resume exploration. Despawned markers are pruned from memory; erasure, eviction and decay invalidate missing targets.

## Shared trails

Carriers and player strokes create the same `Trail { trip, sequence, remaining }`, `Position`, `Allegiance`, `Name`, `EntityKind::Trail` and `MatchEntity`. Both render identically and last 30 simulation seconds, surviving delivery and origin death. Pause freezes decay; speed scales it. Trails store no hidden resource/enemy destination.

Carriers emit every two traveled world units. Sequence numbers retain path metadata for erasing and do not constrain navigation direction. Player strokes retain reversed sequences so the final accepted point is sequence zero. `TrailIds` allocates unique trips and monotonic player marker serials.

`DrawTrailRequests` queues completed strokes; `EraseTrailRequests` queues world locations. Accepted player markers are at least half the configured base vision radius from every earlier accepted point in that stroke (7.5 units). Stat-derived minion vision does not change drawing spacing. Different strokes may overlap; no endpoint is forced.

`PlayerDrawnTrail { serial }` is budget/erasure provenance only. The ten-marker global budget evicts the oldest player marker; carrier markers do not count. Right-click/Erase within two units of a surviving player marker or connecting segment removes its whole trip, including paths with sequence gaps. Erasing works while paused and never removes carrier trails. Drawing requires the living player tower; existing trails survive its loss.

## Validation

Validation is compilation only: native Rust checking and a release WebAssembly build. Do not add or run automated tests or runtime smoke checks. Investigate runtime issues when the user reports them.
