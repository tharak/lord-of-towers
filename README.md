# Lord of Towers

A Rust + Bevy ECS strategy simulation built entirely from Bevy primitive meshes.

**Play:** https://tharak.github.io/lord-of-towers/

[![CI and Pages](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml/badge.svg)](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml)

## Playing

Five towers begin with three empty-equipped minions each. You control Orthanc's orange faction. Minions gather finite elements and fight visible enemies. Towers are destroyed at zero health and drop their stock; the last surviving tower wins.

Click or tap a friendly minion to open a compact grid: three equipment rows, with Fire, Water, Earth, Air and Void buttons in each. A stocked element is enabled on every row. The equipped element has a white outline; clicking it immediately unequips it into cargo for delivery home. Click another available element to request that slot's replacement from the nearest tower. Its stock is reserved immediately and the minion travels home. Changing a choice replaces the pending order. Closing the panel keeps accepted orders. Players no longer draw or erase trails.

Newly gathered elements automatically fill the first empty equipment slot (duplicates allowed). A slot reserved for an incoming tower exchange is skipped. Once no unreserved slot is empty, pickups become cargo and return to the tower. Returned equipment travels as cargo too and may accompany a gathered unit.

Click your tower to select a birth recipe using the element +/− buttons. Exactly five available units create one minion. A primitive marker above the tower indicates that at least five unreserved units are available. NPC towers equip empty slots on nearby minions before automatically consuming the oldest five remaining units to produce minions.

Two fingers pan and pinch to zoom. WASD/arrow keys pan and +/− zoom. Space pauses, 1/2/3 select 1×/2×/4× speed, R restarts, and F3 opens the live ECS inspector. The centered bottom bar contains pause/play symbols, three speeds and restart. The top-right Hide HUD / Show HUD button retains its setting through restart and screen rotation. The web game supports portrait and landscape without page scrolling. Hidden browser tabs suspend simulation.

## Elements, equipment and combat

Thirty seeded deposits contain 5–10 units each, six deposits per element: **fire, water, earth, air and void**. Each deposit is homogeneous and never recharges. A minion carries one unit to a friendly tower. Tower stock appears as five separate vertical donut stacks, one per element, above the beacon. Each donut represents one unit. Minion crowns show carried cargo. Up to three orbiting spheres show equipped units in their element colors. Projectiles and unit bodies retain faction colors.

Every minion has permanent elemental stats starting at **2** each. Five consumed recipe units add matching points only to that newborn. Equipment activates the corresponding bonus: fire damage, water speed, earth maximum health and air vision. Void equipment amplifies these bonuses. For each attribute:

`base × [1 + 0.10 × matching_stat × equipped_copies × (1 + 0.10 × void_stat × equipped_void_copies)]`

Starting and newborn minions have empty slots: damage **10**, speed **4**, maximum health **30**, vision **15**. Void alone gives no bonus. Equipment changes preserve health percentage. Birth stats are separate from physical cargo and equipment.

Minions prioritize visible tower defense, pending tower orders/cargo return, closest visible enemies, visible deposits, freshest unvisited friendly trail, then exploration. Carriers and ordered minions may shoot enemies in range but do not chase them unless defending a tower. Only friendly delivery tower locations are globally known; there is no shared faction vision.

At pickup and every two traveled units, carriers refresh the nearest friendly trail within two units or place a new marker. Markers last 30 simulation seconds. Choosing a marker marks every other visible friendly marker visited for that minion; arrival marks the chosen marker visited. Decay stops while paused and scales with simulation speed.

Shooting costs no resource. A shot pauses the minion for one simulation second and deals damage when its homing sphere reaches the target. Missing or friendly targets invalidate the projectile. Tower circles show fixed radius **15** for territory and defense alerts; there is no energy reserve or recharge.

For every element, **deposits + cargo + tower stock + equipped + reserved + consumed = initial**. Death drops cargo and equipment. Cancelled reservations return to a surviving source tower; when either participant is destroyed, reserved units drop at the recorded source-tower position.

| Faction | Color |
|---|---|
| Orthanc | Orange `#F5901E` |
| Minas Morgul | Blue `#1E83F5` |
| Minas Tirith | Purple `#901EF5` |
| Barad-dur | Green `#0BEF84` |
| Cirith Ungol | Yellow `#EFF51E` |

## Architecture and development

See [architecture](docs/architecture.md), [entity catalog](docs/entity-catalog.md), [gathering contracts](docs/gathering-contracts.md), [equipment contracts](docs/equipment-contracts.md), and the [shared wiki vocabulary](https://github.com/tharak/lord-of-towers/wiki). Gameplay and UI use Bevy ECS; simulation state is independent of mesh and material handles.

For native debugging, install stable Rust and Bevy's Linux windowing dependencies, then use `cargo run --locked`. Browser setup:

```sh
rustup target add wasm32-unknown-unknown
bash scripts/build-web.sh
python3 scripts/serve-web.py
```

Open http://127.0.0.1:4173/lord-of-towers/. The build script installs the wasm-bindgen CLI version recorded in Cargo.lock. WebGL2 needs no server-side application or cross-origin isolation. Each release loads JavaScript and WebAssembly from a commit-specific package directory so browser caches cannot mix game assets across releases.

GitHub Actions checks native Rust compilation with `cargo check --locked` and builds release WebAssembly. Main deploys to GitHub Pages after both succeed. There are no automated unit tests, browser tests, extended simulations, or deployed smoke tests. Runtime issues are investigated when reported by the user. Local compilation runs only when requested.
