# Nomenclature

| Name | Meaning | Code |
|---|---|---|
| **element** | Fire/red, Water/blue, Earth/yellow, Air/green or Void/black units. Typed cargo and tower stock; five FIFO units create a minion. | `ElementInventory` |
| **donut** | One visible element representation, either deposit torus or inventory crown child. | `ElementDeposit / DonutVisual` |
| **torus** | Bevy primitive mesh used for donuts, with no gameplay rules. | `Torus` |
| **tower** | Destructible faction root, delivery destination and circular recharge area. Starts with three minions; consumes five elements per later birth. Fixed radius defaults to 15. | `Tower` |
| **player tower** | Orthanc, the only tower permitting new player strokes. Existing trails survive its destruction. | `PlayerControlled` |
| **tower outline** | Faction-colored primitive circle showing exact tower radius; no gameplay authority. | `TowerInfluenceVisual` |
| **tower beacon** | Sphere atop health cubes, supporting the donut crown. | `TowerBeacon` |
| **stack** | Visual layout of one cube per tower health point or torus per deposit unit. | `Stack / StackVisual` |
| **minion** | Mobile unit with local vision, combat energy and at most one element cargo. | `Minion` |
| **trail** | Shared temporary faction breadcrumb from a carrier or player stroke. Followers choose the highest remaining lifetime among visible unvisited friendly markers; markers expire after 30 simulation seconds unless refreshed by a nearby carrier. | `Trail` |
| **player marker** | Trail provenance for oldest-first ten-marker global budget and whole-stroke erasing. Same navigation/rendering as carrier markers. | `PlayerDrawnTrail` |
| **tower defense alert** | A tower tells friendly minions inside its influence circle to attack its firing enemy, provided the enemy is visible to each recipient. | `TowerDefenseTarget` |
| **combat target** | Selected enemy shared by movement and shooting; closest visible enemy normally, alerted attacker during defense. | `CombatTarget` |
| **energy** | Combat reserve, initially and at most five. Shots cost one; any friendly inclusive tower circle grants one per five-second pulse, once even under overlap. | `EnergyStore / EnergyClock` |
| **energy orb** | Visual sphere for one stored energy point, with no independent reserve. | `EnergyOrbVisual` |
| **energy projectile** | Fired sphere that damages a live hostile target on impact. | `EnergyProjectile` |
| **vision** | Inclusive minion detection circle, base 15 and initially 18.6 after air/void bonuses; only friendly delivery towers are globally known. | `Vision` |
| **entity kind** | Application category paired with Name and visible to the live inspector. | `EntityKind` |

A component stores entity state; a resource stores shared state. Visual children hold no gameplay inventory. For each element, deposits + cargo + tower stock + consumed equals its initial total.

Elemental stats start at two each. Fire modifies damage, water speed, earth maximum health, air vision, and void multiplies the other bonuses. Each matching point adds 10% of its base stat, multiplied by (1 + 10% × void points). Newborn recipes add one matching stat point per consumed unit; older minions do not change.
