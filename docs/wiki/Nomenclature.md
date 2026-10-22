# Nomenclature

| Name | Meaning | ECS / mesh |
|---|---|---|
| element | Physical Fire, Water, Earth, Air or Void unit. Fire is red, Water blue, Earth yellow, Air teal, Void gray in the current palette. | `Element` |
| inventory | Ordered deque of actual units in deposit, cargo, tower stock or reservation. | `ElementInventory` |
| donut | Visible stock, cargo or deposit unit. | `DonutVisual` / stack child |
| torus | Bevy primitive mesh used to render a donut. Not a resource type. | `Torus` |
| cargo | Gathered overflow element or unequipped units being carried home. | Minion `ElementInventory` |
| equipment | Up to three individually slotted elements, duplicates allowed; each grants 30% of its base attribute, with 10% bonus amplification per Void. | `EquippedElements` |
| equipment sphere | Colored visual for one equipped slot; separate from cargo crown. | `EquippedElementVisual` / sphere |
| elemental stat | Permanent numeric birth point, initially two each plus recipe counts; not an inventory unit. | `ElementStats` |
| reservation | Element removed immediately from tower stock for a pending exchange. | Order `ElementInventory` |
| tower order | Travel to source tower for an equip, equipment return or cargo delivery. | `EquipmentOrder` |
| recipe | Exactly five physical units consumed to create a minion and increase its birth stats. | `PlayerCommand::CreateMinion` / NPC FIFO |
| player tower | Orthanc; controlled through contextual selection and manual recipes. | `PlayerControlled` |
| production indicator | Primitive marker visible at five available player-tower units. | `ProductionIndicator` |
| tower | Destructible structure shown by stacked health cubes and beacon sphere. | `Tower` |
| beacon | Sphere above tower health stack; donuts sit above it. | `TowerBeacon` |
| minion | Mobile combat/gathering unit with local vision, cargo and three equipment slots. | `Minion` |
| trail | Fading friendly carrier breadcrumb, refreshed within two units; no player drawing. | `Trail` |
| projectile | Free homing faction-colored shot; damage on impact. | `Projectile` / sphere |
| influence circle | Fixed tower territory/defense-alert radius; does not recharge minions. | `InfluenceEmitter` / `TowerInfluenceVisual` |
| crown | Donut arrangement over a minion or tower beacon, one per unit. Tower stock has five separate vertical stacks, one per element; minion cargo uses crown tiers. | `DonutVisual` |
| stack | Generic count-driven visual arrangement for tower health and deposit quantities. | `Stack` / `StackVisual` |
| HUD | Hideable title, tower summaries and bottom simulation controls. Contextual selection panel is separate. | `HudRoot` |

Energy stores, energy recharge, energy projectiles and player-drawn trails are removed. Use the names above for new work.
