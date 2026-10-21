# Nomenclature

| Name | Meaning | Code |
|---|---|---|
| **rawElement** | Finite gameplay resource. A deposit holds 5–10 units; a minion can carry one; a tower stores delivered units and spends five per new minion. | `RawElement` component |
| **donut** | A visible object representing rawElement. A world donut is a collectible deposit; a crown donut is a visual child above a carrier or stocked tower. One crown donut represents exactly one unit. | `RawElementDeposit` or `DonutVisual` entity kind |
| **torus** | The Bevy primitive mesh used to draw donuts. It has no gameplay rules. | `Torus` mesh |
| **tower** | A capturable faction-owned root that supplies energy, holds rawElement stock, and spawns minions. | `Tower` component |
| **minion** | A mobile faction unit with vision, combat energy, a gathering route, and at most one carried rawElement. | `Minion` component |
| **trail** | A temporary faction-owned breadcrumb left by a carrier. Visible allies follow decreasing sequence numbers toward a deposit. | `Trail` entity/component |
| **link** | A live tower-rooted supply edge that transfers combat energy every five simulation seconds. Links do not share vision or rawElement. | `Link` entity/component |
| **energy** | Combat/forwarding reserve. An attack costs one; towers are unlimited supply roots. It is separate from rawElement. | `EnergyStore` and `EnergyClock` |
| **energy orb** | A small sphere around a minion, representing one stored energy point. It carries no gameplay energy itself. | `EnergyOrbVisual` entity kind; Bevy `Sphere` mesh |
| **energy projectile** | A fired energy point moving toward a live hostile target. Damage happens on impact. | `EnergyProjectile` entity/component; Bevy `Sphere` mesh |
| **vision** | A minion's 15-unit circular detection range for deposits, trails, and enemies. Friendly tower locations remain known for delivery. | `Vision` component |
| **entity kind** | Inspectable application category assigned with `Name` to every application-created entity. | `EntityKind` component |

A **component** stores state on one ECS entity. A **resource** stores match-wide state. A **visual child** is an entity attached to a parent and has no inventory of its own.
