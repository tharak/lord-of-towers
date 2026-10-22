# Donut and crown presentation contracts

An **element** is a typed gameplay unit: Fire/red, Water/blue, Earth/yellow, Air/green or Void/black. A **donut** is its visible representation; a **torus** is the Bevy primitive mesh. `ElementInventory` is the only inventory authority.

`ElementDeposit` stacks show one torus per unit of their homogeneous inventory. Tower/minion crowns show one `DonutVisual` child per actual stored/carried unit, including unequipped elements traveling home, following inventory order. Shared element materials color both existing and newly created pieces when inventory contents change. Tower/minion bodies, trails and projectiles retain faction colors. Equipped element spheres use element colors and are separate from cargo donuts.

Crown children have Name, EntityKind::DonutVisual, DonutVisual, ChildOf, mesh/material and transform. They hold no inventory or elemental stats. Eight pieces per tier sit above minions or above the tower health stack and beacon. Reconciliation removes excess children after delivery, production or inventory loss; parent despawn removes all children.

`ElementStats` belongs to the minion and remains independent of its cargo crown. The F3 inspector shows inventory, equipped slots, reservations, five birth stats and derived attributes. A small contextual panel manages equipment and tower recipes; there is no persistent equipment HUD. Combat priorities follow the current [gathering contracts](gathering-contracts.md).

Validation is compilation only. Automated tests and runtime smoke checks are not added or run.
