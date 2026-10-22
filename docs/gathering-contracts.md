# RawElement and trail contracts

`GatheringPlugin` owns finite deposits, local navigation, collection/delivery and trail decay/emission. `TrailInputPlugin` owns player gestures and placement. Shared state lives in `model.rs`; meshes and materials belong to presentation.

## Inventory and reset

`RawElement(u32)` is deposit quantity, minion cargo (0–1), or retained tower stock. `RawElementEconomy { initial }` records only the starting total. The invariant is **remaining deposits + carried cargo + tower stock = initial**. There is no spending or resource regeneration.

Setup creates thirty seeded deposits of 5–10 units, five empty towers and exactly three minions per tower. No more minions spawn. Collection removes one deposit unit, despawning an empty deposit. Delivery adds cargo to stock and clears the carrier's emitter. Death drops minion cargo or tower stock into a real neutral deposit. Inventory changes are immediate and deterministic, ordered by distance and stable `SimulationId` ties where needed.

`spawn_deposits(commands, config)` resets gathering randomness, `TrailIds`, both request queues and the economy. Match restart removes all `MatchEntity` deposits, minions, towers and trails before seeded recreation. `drop_cargo` creates match-owned deposits with stable drop IDs. Presentation children never own inventory.

## Local navigation

Priority is cargo return → visible deposit → visible friendly trail → seeded exploration. Vision is an inclusive circle, default radius 15, and applies to acquisition and continued pursuit. Friendly tower positions are globally known solely for delivery. There is no remote resource or enemy knowledge and no shared faction vision. Empty energy does not change gathering.

A collector chooses the nearest friendly tower and retains it until missing or hostile, then selects another. With no friendly tower it explores while keeping cargo. Pickup radius defaults to one; delivery radius to three. Exploration retargets on arrival or every three simulation seconds, within ten world units and map bounds.

`GatheringState` is Exploring, Following { trip, sequence }, or Returning { tower }. A joining minion selects a visible friendly marker. It follows strictly decreasing sequence numbers after reaching each marker, including gaps among surviving markers. Only visible continuation is eligible. Missing continuation returns it to exploration. Private `VisitedTrails` prevents repeatedly joining an exhausted trip; removed trips clear its memory. Erasure, eviction and decay invalidate missing navigation markers and reset retargeting.

## Shared trails

Both sources create the same `Trail { trip, sequence, remaining }`, `Position`, `Allegiance`, `Name`, `EntityKind::Trail` and `MatchEntity`. They share rendering and the default 30-simulation-second lifetime. They survive delivery, carrier death and origin tower death. Pause freezes decay; speed scales it. Metadata stores no hidden deposit or enemy target.

Carriers start sequence zero at collection, then emit every two traveled world units, increasing sequence toward the tower. Followers therefore travel back toward the source. Player strokes reverse their sequences so the final accepted point has sequence zero. `TrailIds { next_trip, next_player_marker }` allocates unique trips for both producers and monotonic player marker serials.

`DrawTrailRequests(Vec<Vec<Vec2>>)` queues completed strokes; `EraseTrailRequests(Vec<Vec2>)` queues world locations. Private `DrawingTrail` retains bounded raw pointer samples. Accepted marker points must be at least half `vision_radius` from **all** previously accepted points in that stroke (7.5 by default). Nonfinite points are ignored; no endpoint is forced. Different strokes may overlap.

`PlayerDrawnTrail { serial }` is provenance for budget and erasure only. At most `drawn_trail_capacity` live player markers exist globally (default ten). Each new marker evicts the oldest serial when full; carrier markers do not count. Erase/right-click within two world units of a surviving player marker or segment removes its whole trip. Segments connect consecutive surviving sequence values, including gaps. Erasure never removes carrier trails and works while paused. Drawing requires a living `PlayerControlled` tower; existing markers persist after its loss.

## Validation

CI covers inclusive vision, local pursuit and target loss, decreasing sequence/gaps, cargo priority, deterministic collection, finite deposits, death drops, restart, shared trip uniqueness, common lifetime, spacing across an entire stroke, global oldest eviction, segment erasure and stale navigation cleanup. Conservation and bounded entity counts apply throughout extended simulations. Tests/builds run in GitHub Actions; formatting is allowed locally.
