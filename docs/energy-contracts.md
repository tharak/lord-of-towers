# Energy and tower area contracts

`EnergyPlugin` owns area recharge in `SimSet::Energy`, after combat, gathering and tower growth and before influence/victory. Shared components are defined in `model.rs`; energy simulation has no mesh or material handles.

## Recharge

Every tower starts with radius `tower_influence_radius` (15). Retained `RawElement` stock adds `tower_radius_per_raw` (one) per unit. `InfluenceEmitter.radius` is updated from this formula. A minion inside or exactly on any living friendly tower circle gains one energy per pulse, up to `EnergyStore.capacity` (five). Its faction comes from `Allegiance`.

A pulse is `energy_tick_seconds * energy_recharge_ticks`, default 1 × 5 simulation seconds. `EnergyClock` accumulates scaled fixed time and counts completed pulses. Multiple due pulses are applied sequentially. Pause, hidden tabs and victory freeze the clock; speed scales it. Restart resets the clock and minion energy to the configured starting value, default five.

Eligibility uses the exact inclusive circle test against live friendly towers. Multiple friendly circles still grant only one energy per minion per pulse. Enemy influence and contested territory never block recharge. Full minions remain full; there is no transfer between minions, topology registry, forwarding budget or visual edge entity. Tower destruction immediately removes that tower's eligibility area.

## Combat and presentation

An eligible shot spends `attack_energy_cost` (one), creates an `EnergyProjectile` and starts a one-simulation-second `ShootingPause`. Damage is applied on impact; impacts in one fixed step resolve together. Projectiles survive shooter death and fizzle on target loss, target becoming friendly, timeout or impact. Insufficient energy causes no shot or cost. Gathering and movement continue at zero energy except during the shooting pause.

One `EnergyOrbVisual` sphere child represents each stored minion energy point. An independent `TowerInfluenceVisual` child with `EntityKind::TowerInfluenceVisual` renders a faction-colored primitive circular outline at the exact tower radius. It updates with stock/radius and despawns with its parent. These children contain no energy, inventory or recharge authority.

## Validation

CI tests inclusive boundaries, friendly ownership, growth from retained stock, overlap granting once, contested-area eligibility, capacity, multiple pulses, pause/speed and restart. Combat tests verify energy cost and projectile lifecycles. Routine local builds/tests are not run; formatting is allowed.
