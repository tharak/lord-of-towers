# Lord of Towers

A Rust + Bevy ECS strategy simulation built entirely from Bevy primitive meshes.

**Play:** https://tharak.github.io/lord-of-towers/

[![CI and Pages](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml/badge.svg)](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml)

## Playing

Five towers begin with three minions each and zero rawElement stock. These fifteen minions are the entire starting population; no additional minions spawn. Minions gather finite resources and fight nearby rivals. Enemy minions take priority over towers in attack range. Towers are destroyed at zero health, dropping their stock. The last surviving tower wins.

You control Orthanc's orange faction. Drag the left mouse button or a finger in **Draw** mode to leave a trail; click or tap for one marker. Friendly empty-handed minions discover trails through their own vision and follow them toward the stroke end. They seek visible deposits first. Carriers continue returning cargo. Drawing assigns no minion and stores no enemy target. Orthanc's death prevents new drawing, while existing trails last until expiry or erasure.

A stroke accepts markers at least half a vision radius apart from every previously accepted point in that stroke: 7.5 world units by default. Separate strokes may overlap. At most ten player markers exist globally; every new marker replaces the oldest when necessary. Right-click, or tap in **Erase** mode, within two world units of a marker or connecting segment to remove its entire surviving stroke. Carrier trails cannot be erased this way.

Use two fingers to pan and pinch to zoom; lift every finger before drawing again. WASD/arrow keys pan and +/− zoom. Space pauses, 1/2/3 select 1×/2×/4× speed, R restarts the seed, and F3 opens the live ECS inspector. The centered seven-button bar provides pause/play, three speeds, restart, Draw and Erase. Pause/play uses rectangular UI shapes. A standalone top-right **Hide HUD / Show HUD** button hides or restores the HUD, retaining the choice through restart and screen rotation. F3 opens the inspector; there is no Entities button. The battlefield fills portrait and landscape viewports without page scrolling. Hidden browser tabs suspend simulation.

## Resources, territory and energy

Thirty seeded deposits contain 5–10 rawElement each. Deposits never recharge. A minion carries one unit to a friendly tower; death drops cargo and tower destruction drops stock. Delivered stock is retained and grows the tower's influence radius: **15 + stock** world units by default. It is never spent. Remaining deposits + carried cargo + tower stock always equal the initial resource total.

Each minion sees a 15-unit circle. It seeks visible deposits, then visible friendly trails, otherwise explores. Only friendly delivery towers are globally known. There is no shared faction vision. Carriers leave a marker every two traveled units; player and carrier markers use the same `Trail` component, rendering, navigation and 30-simulation-second lifetime. Followers move through decreasing sequence numbers. Trail decay stops while paused and scales with speed.

Territory samples current radial influence, with soft borders and contested or neutral ground. Each tower has a faction-colored circular outline showing its exact current radius. Inside or on a friendly tower's circle, a minion gains one energy every five simulation seconds, up to capacity five. Overlapping friendly circles recharge a minion only once per pulse; enemy influence or contested ground does not block recharge.

An attack spends one energy and fires a sphere; damage happens on impact. A minion pauses movement for one simulation second after firing. Shots fizzle when targets disappear or become friendly. Minions gather even at zero energy. One orbiting sphere represents each stored energy point. Donut crowns represent actual rawElement inventory; tower cubes represent health. Neither visual children nor mesh dimensions determine gameplay rules.

| Faction | Color |
|---|---|
| Orthanc | Orange `#F5901E` |
| Minas Morgul | Blue `#1E83F5` |
| Minas Tirith | Purple `#901EF5` |
| Barad-dur | Green `#0BEF84` |
| Cirith Ungol | Yellow `#EFF51E` |

## Architecture and development

See [architecture](docs/architecture.md), [entity catalog](docs/entity-catalog.md), [gathering contracts](docs/gathering-contracts.md), [energy contracts](docs/energy-contracts.md), and the [shared wiki vocabulary](https://github.com/tharak/lord-of-towers/wiki). Gameplay and UI use Bevy ECS; simulation state is independent of mesh and material handles.

For native debugging, install stable Rust and Bevy's Linux windowing dependencies, then use `cargo run --locked`. Browser setup:

```sh
rustup target add wasm32-unknown-unknown
bash scripts/build-web.sh
python3 scripts/serve-web.py
```

Open http://127.0.0.1:4173/lord-of-towers/. The build script installs the wasm-bindgen CLI version recorded in Cargo.lock. WebGL2 needs no server-side application or cross-origin isolation.

Routine builds and tests run in GitHub Actions. PR checks include formatting, Clippy, headless gameplay tests, release WASM and Chromium/Firefox smoke tests. Main deploys after checks pass, followed by a live Pages smoke test. Extended simulation runs check bounded entities and resource conservation. Browser failures retain screenshots and traces. Local formatting is allowed; local builds/tests are reserved for reproducing CI failures when requested by the lead.
