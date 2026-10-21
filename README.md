# Lord of Towers

A Rust + Bevy ECS strategy simulation. Five towers spawn armies, project territory through their presence, fight, and can be destroyed. All art is made from Bevy primitive meshes.

**Play:** https://tharak.github.io/lord-of-towers/

[![CI and Pages](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml/badge.svg)](https://github.com/tharak/lord-of-towers/actions/workflows/ci.yml)

## Controls

Use the on-screen buttons or Space to pause, 1/2/3 for 1×/2×/4×, R to restart, and F3 to inspect entities. WASD or arrow keys pan; +/− zoom. Click the canvas if keyboard controls are not focused. The simulation pauses while the tab is hidden. Restart repeats the configured seed.

The inspector queries the live ECS world. Cycle kind/faction filters, include engine entities, navigate entities, and page through their component names. It does not store a second entity database.

## Simulation

Each tower begins as its own faction. Each tower starts with three minions and no rawElement stock. Five delivered rawElement create one new minion, capped at 100 living minions per origin tower; stock is retained while capped. Minions gather and attack nearby rivals. Enemy minions take priority over towers when both are in attack range. Combat damage occurs when fired energy spheres hit; impacts in one fixed step resolve together. Towers are destroyed at zero health and drop their stock. The last tower standing wins.

You control Orthanc, the orange tower. Drag with the left mouse button or draw with a finger to leave an attack trail; a click or tap leaves one marker. No minion is assigned when you draw. Orthanc's empty-handed minions discover nearby markers through their vision, follow the trail toward its end, and attack enemies in range. A trail ending on an enemy follows that target while it remains hostile; a trail ending on open ground holds its followers there. Minions carrying rawElement keep delivering it. Right-click near any marker to erase its whole attack trail, including while paused; right-clicking empty ground does nothing. If Orthanc is destroyed, you cannot place new attack trails through another tower.

Territory reflects current radial influence, with soft borders and neutral ground where no army or tower is present. It is not permanent paint or a tile movement system. The initial numerical balance is intentionally simple and configurable in `SimulationConfig`.

| Faction | Color |
|---|---|
| Orthanc | Orange `#F5901E` |
| Minas Morgul | Blue `#1E83F5` |
| Minas Tirith | Purple `#901EF5` |
| Barad-dur | Green `#0BEF84` |
| Cirith Ungol | Yellow `#EFF51E` |

## RawElement, vision and trails

Thirty neutral deposits each start with 5–10 rawElement, determined by the match seed. Each unit is shown as one donut in a vertical stack. Minions carry one unit per trip and deliver to the closest friendly tower selected when collecting. Exhausted stacks disappear and never recharge. Minion death drops carried rawElement; tower destruction drops remaining stock as a collectable deposit.

Every minion has a 15-unit circular field of vision. It seeks visible deposits first, then follows visible friendly trails toward their source; otherwise it explores randomly. Only friendly tower locations are globally known for delivery. Resource, enemy and trail knowledge is not shared between minions. The spectator can still see the entire map.

Carriers leave faction-colored trail markers every two world units. These fade over 30 simulation seconds and remain after delivery or death. Followers traverse only visible markers toward the source, never obtaining remote deposit locations from trail metadata.

One visible donut per rawElement unit forms a crown above each carrying minion or stocked tower, stacked in eight-piece tiers. Towers use vertical stacks of cubes, one per health point, with a sphere beacon and donut crown above. The debug HUD still shows numeric stock; the inspector shows exact inventory, gathering state, vision radius and trail lifetime. Pause freezes trail decay, speed scales it, and restart restores the original deposit quantities and three minions per tower. Supplies are finite for the match.

## Energy and supply

Minions start with five energy and can store at most five. One small sphere circles each minion per stored energy point. Attacking spends one energy immediately and fires a sphere at the target; damage occurs when it hits. Minions stop for one simulation second when they shoot, and their longer attack range lets them defend an ordered position. Shots fizzle if their target disappears or becomes friendly. Gathering continues at zero energy; donuts never provide combat energy.

Straight faction-colored lines are actual Link entities. They connect friendly minions into tower-rooted branches, with at most 15 world units per link. Every five simulation seconds, each tower or connected minion sends one energy total to an eligible child, rotating fairly. Towers supply unlimited energy; minions spend their stored energy, including their last point. Transfers happen simultaneously, so incoming energy cannot travel multiple links in one pulse. Full recipients are skipped. Links do not share vision.

## Architecture and collaboration

See [ECS contracts](docs/architecture.md), [entity catalog](docs/entity-catalog.md), [shared nomenclature on the GitHub wiki](https://github.com/tharak/lord-of-towers/wiki), [gathering contracts](docs/gathering-contracts.md), and the [implementation tickets](https://github.com/tharak/lord-of-towers/milestone/6). Simulation, orders, gathering, energy, influence, presentation and inspection are separate plugins. Components hold gameplay state; systems implement behavior. Meshes/materials live only in presentation, so models can be replaced without changing gameplay ranges or rules.

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

Visual quality can be reviewed directly on Pages. No custom models, external textures, multiplayer, persistence, or mobile controls are included in this prototype.
