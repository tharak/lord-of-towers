# Lord of Towers: shared vocabulary

These pages define the current Bevy ECS game contracts for issues, reviews and presentation changes.

- [Nomenclature](Nomenclature): resources, towers, minions, shared trails and area recharge.
- [Entities](Entities): every application kind and lifecycle.
- [Components and Resources](Components-and-Resources): gameplay state, private visual roles and shared resources.

Five towers start with three minions each. Towers consume any five delivered elements in FIFO order to create a minion; consumed types raise that newborn's corresponding elemental stats above the starting two points. Tower radius stays fixed at 15. Friendly tower circles recharge combat energy. Player and carrier markers share the same temporary Trail behavior; followers prefer the marker with the highest remaining lifetime. Towers alert nearby friendly minions when fired upon. Movement priorities are defense, cargo return, visible enemies, element deposits, trails, then exploration.

The live inspector queries ECS directly and is opened with F3. It stores no second entity registry. Keep these pages aligned with the repository [entity catalog](https://github.com/tharak/lord-of-towers/blob/main/docs/entity-catalog.md), [gathering contracts](https://github.com/tharak/lord-of-towers/blob/main/docs/gathering-contracts.md) and [energy contracts](https://github.com/tharak/lord-of-towers/blob/main/docs/energy-contracts.md).

Validation is compilation only: native Rust checking and a release WebAssembly build. Pages deploys after both succeed. Automated tests and runtime smoke checks are removed; runtime issues are investigated when reported by the user.
