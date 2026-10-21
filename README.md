# Lord of Towers

A Rust + Bevy ECS spectator simulation. Five towers spawn armies, project territory through their presence, fight, and capture each other. All art is made from Bevy primitive meshes.

**Play:** https://tharak.github.io/lord-of-towers/

[![CI and Pages](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml/badge.svg)](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml)

## Controls

Use the on-screen buttons or Space to pause, 1/2/3 for 1×/2×/4×, R to restart, and F3 to inspect entities. WASD or arrow keys pan; +/− zoom. Click the canvas if keyboard controls are not focused. The simulation pauses while the tab is hidden. Restart repeats the configured seed.

The inspector queries the live ECS world. Cycle kind/faction filters, include engine entities, navigate entities, and page through their component names. It does not store a second entity database.

## Simulation

Each tower begins as its own faction. Towers spawn one minion every two seconds, capped at 100 living minions per origin tower. Minions move freely, spread toward the frontline and attack nearby rivals. Combat damage is simultaneous. Captured towers restore health and begin producing for their conqueror; existing armies keep their original faction. Controlling all five towers wins.

Territory reflects current radial influence, with soft borders and neutral ground where no army or tower is present. It is not permanent paint or a tile movement system. The initial numerical balance is intentionally simple and configurable in `SimulationConfig`.

| Faction | Color |
|---|---|
| Orthanc | Orange `#E69F00` |
| Minas Morgul | Green `#009E73` |
| White Tower of Minas Tirith | Sky blue `#56B4E9` |
| Barad-dûr | Vermilion `#D55E00` |
| Tower of Cirith Ungol | Purple `#CC79A7` |

## Energy and supply

Thirty neutral torus-shaped pickups are scattered using the match seed. Each holds one energy, stays in place when collected, and recharges after five simulation seconds. Bright pickups are ready; dim ones are recharging.

Minions start with five energy and can store at most five. Each successful attack costs one. Empty minions seek nearby charged pickups or return toward a friendly supply network; movement and territory influence remain active.

Straight faction-colored lines are actual Link entities. They connect friendly minions into tower-rooted branches, with at most 15 world units per link. Every five simulation seconds, each tower or connected minion sends one energy total to an eligible child, rotating fairly. Towers supply unlimited energy; minions spend their stored energy, including their last point. Transfers happen simultaneously, so incoming energy cannot travel multiple links in one pulse. Full recipients are skipped.

Inspect minions for energy reserves, pickups for recharge state, and links for endpoint IDs. Pause and speed controls affect energy timers; restart resets all energy state and recreates the same pickup positions. See the [energy milestone](https://github.com/tharak/lord-of-towers/milestone/2) and [energy ECS contracts](docs/energy-contracts.md).

## Architecture and collaboration

See [ECS contracts](docs/architecture.md), [entity catalog](docs/entity-catalog.md), and the [implementation tickets](https://github.com/tharak/lord-of-towers/milestone/1). Simulation, energy, influence, presentation and inspection are separate plugins. Components hold gameplay state; systems implement behavior. Meshes/materials live only in presentation, so models can be replaced without changing gameplay ranges or rules.

## Development

Install stable Rust. For native debugging, install Bevy's Linux windowing dependencies and run `cargo run --locked`. For the browser:

```sh
rustup target add wasm32-unknown-unknown
bash scripts/build-web.sh
python3 scripts/serve-web.py
```

Open http://127.0.0.1:4173/lord-of-towers/. The script installs the wasm-bindgen CLI version recorded in Cargo.lock. Desktop WebGL2 browsers are the initial target; no server or cross-origin isolation is needed.

## Testing and releases

Routine testing runs in **GitHub Actions**, not on developer machines. Every PR checks formatting, Clippy, headless gameplay tests, a release WASM build, and Chromium/Firefox smoke tests. Main deploys only after those checks pass; a further smoke test checks the actual Pages URL. Main, scheduled, and manual runs include an extended population-bound simulation. Browser failures retain screenshots and traces as workflow artifacts.

Visual quality can be reviewed directly on Pages. No custom models, external textures, multiplayer, persistence, mobile controls or economy are included in this prototype.
