# Energy network contracts (historical pickup design superseded)

Historical implementation contract: tower capture references below are superseded by tower destruction in the current simulation. See the [entity catalog](entity-catalog.md).

The rawElement implementation supersedes all pickup/recharge, energy-seeking AI, timed spawning and pickup browser hooks below. Current gathering interfaces are in [gathering contracts](gathering-contracts.md). Network topology, energy costs, pulse snapshots and fairness remain unchanged.

# Energy and supply contracts

Lead owns model.rs and this contract; coordinate changes before editing shared interfaces. Tests run in GitHub Actions. No local test/build execution; rustfmt allowed.

## Ownership

- Simulation agent: src/simulation.rs and src/energy/pickups.rs.
- Networking agent: src/energy/mod.rs and src/energy/network.rs.
- Presentation agent: src/presentation.rs, src/inspector.rs and src/main.rs.
- Lead: model.rs, entity catalog, README, browser tests, integration.

## Components and phase

Shared model.rs defines SimulationId, EnergyStore, EnergyPickup, Link, SupplyConnection, ForwardingCursor and EnergyClock. Stable IDs for towers/minions retain existing sequence (0..5 towers, minions from 5). Pickups use a separate high namespace starting at 1 << 63. Link endpoints refer to live entities; links need no gameplay SimulationId.

SimulationPlugin removes its private SimulationId, inserts SimSet::Energy after Capture and before Influence, and creates EnergyStore + ForwardingCursor on all spawned minions and ForwardingCursor on towers. Initial energy is min(initial_energy, energy_capacity). Simulation create_match calls crate::energy::pickups::spawn_pickups(commands, config) with a separate seeded RNG. MatchEntity marks every pickup and link. Restart resets optional EnergyClock and removes all MatchEntity entities before presentation. Capture resets the captured tower's ForwardingCursor.

EnergyPlugin (energy/mod.rs) initializes EnergyClock and registers the ordered FixedUpdate chain in SimSet::Energy: recharge_pickups -> collect_pickups -> reconcile_links -> transfer_energy. These system functions are pub(super) in their submodules; spawn_pickups is public. The plugin does not spawn pickups independently. Simulation tests can run without EnergyPlugin but create_match still creates pickups. Full feature tests install both plugins.

Each energy tick is one simulation second; a pulse is every energy_tick_seconds * energy_transfer_ticks (default five seconds). EnergyClock.elapsed accumulates simulation_delta and pulses counts completed pulses. Pause/hidden/victory gate the SimSet; speed scales simulation_delta. Energy phase follows combat/capture, so incoming energy can fund subsequent attacks. Reconcile after capture/death, with deferred effects flushed before transfers.

## Pickups and AI

30 neutral permanent pickups, available at setup. Each holds one unit and recharges in place five seconds after collection. Seed positions independently from unit RNG, inside map with margin, avoid towers and overlapping pickups (bounded placement attempts). Radius for collection is 1.0. Process pickups by stable ID; collectors by distance then stable ID. Full minions don't consume pickups. Collector inventory is updated immediately so multi-pickup collection cannot exceed capacity. Depleted pickups stay alive and visible.

An empty minion overrides combat targeting and stopping-in-melee behavior. Seek nearest available pickup within energy_seek_radius=15; otherwise nearest friendly tower or minion with SupplyConnection (validate the connection root is still a live allied tower). Resume normal AI immediately after replenishment. If no supply/pickup candidate exists, remain at current position; it can reacquire a candidate on later updates. It still contributes influence. Successful attacks cost attack_energy_cost=1; absent target, cooldown, or insufficient energy cause no cost.

## Networking

At each energy phase construct a minimum-distance forest separately per faction, starting with all its towers as connected roots. Repeatedly connect the closest not-yet-connected minion to a connected node within link_range=15; deterministic edge tie: distance, source stable ID, target stable ID. All towers are roots; no tower-to-tower edges. Use O(n^2) Prim-style candidate updates per faction, not repeated all-pairs scans per added edge. Each connected minion has one incoming edge and can have multiple children; isolated minions have no SupplyConnection or forwarding rights. Preserve unchanged Link entities (endpoint pair + allegiance), remove stale/changed ones, create missing ones with Name, EntityKind::Link, MatchEntity, Allegiance(source faction), Link. No separate authoritative graph registry.

Each sender sends at most ONE total per pulse to a child with capacity. Towers have unlimited supply; minions must have energy at pulse start and spend one, including their last. Recipient eligibility and sender affordability use a pulse-start snapshot; receiving cannot finance same-pulse forwarding and a full recipient is skipped even if it will also send. Children ordered by SimulationId; choose first after ForwardingCursor.last_recipient, wrap around. Advance cursor only for an actual transfer, preserve across rewiring. Process multiple due pulses sequentially if needed; every pulse has its own snapshot. Energy capacity always respected.

## Presentation and browser hooks

Engine Torus mesh for neutral pickups, bright charged and dim depleted. Shared thin Cuboid mesh stretches directly between endpoint positions, faction material; it remains a separate Link entity. Update endpoints as minions move and guard missing endpoints. Links carry no combat or influence components. Keep links slightly above ground and use ASCII labels compatible with embedded font.

Inspector adds EnergyPickup/Link kinds, EnergyStore current/capacity, pickup availability/recharge, SupplyConnection parent/root and Link endpoint IDs. Browser #game-status adds data-energy-pickups, data-charged-pickups, data-links, data-total-minion-energy, data-energy-pulses. Existing hooks/controls retained. main installs EnergyPlugin after SimulationPlugin.
