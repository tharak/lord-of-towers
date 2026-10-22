# Lord of Towers

A Rust/Bevy ECS strategy simulation using only engine primitive meshes. [Play in the browser](https://tharak.github.io/lord-of-towers/).

- [Nomenclature](Nomenclature): agreed gameplay and mesh names.
- [Entities](Entities): every entity kind and lifecycle.
- [Components and Resources](Components-and-Resources): shared ECS contracts.

You control Orthanc. Select a friendly minion for three rows of five element buttons. Stocked types are enabled in all rows; an outlined equipped button immediately unequips that unit into cargo for delivery home. Pickups fill empty unreserved equipment slots automatically. Exchanges reserve stock immediately and require traveling home. Select the player tower for one matching element-button row. Each click adds one available unit; the fifth selection creates an empty-equipped minion and clears the selection. A sphere indicates five available units. NPCs auto-equip nearby empty slots before automatic FIFO births.

Permanent birth stats start at two each; recipes add matching points. Equipped fire/water/earth/air activate damage/speed/health/vision bonuses and equipped void amplifies them. Duplicate gear is allowed. Element-colored spheres mean equipment; donut crowns mean cargo/stock. Shooting is free, pauses movement for one second, and damages on projectile impact. There is no energy reserve/recharge or player trail drawing.

Carriers create and refresh temporary trails; followers prefer the freshest visible unvisited friendly marker. Defense overrides homeward travel. Towers have fixed radius 15, are destroyed at zero health, and the last surviving tower wins. F3 inspects live ECS. Hide HUD and two-finger mobile camera controls remain available.

See repository [equipment contracts](https://github.com/tharak/lord-of-towers/blob/main/docs/equipment-contracts.md). Validation is native/web compilation only; no automated tests or runtime smoke checks.
