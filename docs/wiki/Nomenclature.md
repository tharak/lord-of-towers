# Nomenclature

| Name | Meaning | Code |
|---|---|---|
| **rawElement** | Finite resource in deposits, one-unit minion cargo and retained tower stock. Stock grows tower radius and is never spent. | `RawElement` |
| **donut** | One visible rawElement representation, either deposit torus or inventory crown child. | `RawElementDeposit / DonutVisual` |
| **torus** | Bevy primitive mesh used for donuts, with no gameplay rules. | `Torus` |
| **tower** | Destructible faction root, delivery destination and circular recharge area. Starts with three minions; no later spawning. Radius defaults to 15 + stock. | `Tower` |
| **player tower** | Orthanc, the only tower permitting new player strokes. Existing trails survive its destruction. | `PlayerControlled` |
| **tower outline** | Faction-colored primitive circle showing exact tower radius; no gameplay authority. | `TowerInfluenceVisual` |
| **tower beacon** | Sphere atop health cubes, supporting the donut crown. | `TowerBeacon` |
| **stack** | Visual layout of one cube per tower health point or torus per deposit unit. | `Stack / StackVisual` |
| **minion** | Mobile unit with local vision, combat energy and at most one rawElement cargo. | `Minion` |
| **trail** | Shared temporary faction breadcrumb from a carrier or player stroke. Followers use visible decreasing sequences; all markers expire after 30 simulation seconds. | `Trail` |
| **player marker** | Trail provenance for oldest-first ten-marker global budget and whole-stroke erasing. Same navigation/rendering as carrier markers. | `PlayerDrawnTrail` |
| **energy** | Combat reserve, initially and at most five. Shots cost one; any friendly inclusive tower circle grants one per five-second pulse, once even under overlap. | `EnergyStore / EnergyClock` |
| **energy orb** | Visual sphere for one stored energy point, with no independent reserve. | `EnergyOrbVisual` |
| **energy projectile** | Fired sphere that damages a live hostile target on impact. | `EnergyProjectile` |
| **vision** | Inclusive minion detection circle, default 15; only friendly delivery towers are globally known. | `Vision` |
| **entity kind** | Application category paired with Name and visible to the live inspector. | `EntityKind` |

A component stores entity state; a resource stores shared state. Visual children hold no gameplay inventory. Remaining deposits + carried cargo + tower stock equals the initial rawElement total.
