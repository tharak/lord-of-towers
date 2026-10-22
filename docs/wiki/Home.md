# Lord of Towers: shared vocabulary

These pages define the current Bevy ECS game contracts for issues, reviews and presentation changes.

- [Nomenclature](Nomenclature): resources, towers, minions, shared trails and area recharge.
- [Entities](Entities): every application kind and lifecycle.
- [Components and Resources](Components-and-Resources): gameplay state, private visual roles and shared resources.

Five towers start with three minions each; no additional minions spawn. Retained rawElement grows tower radius. Friendly tower circles recharge combat energy. Player and carrier markers share the same temporary Trail behavior.

The live inspector queries ECS directly and is opened with F3. It stores no second entity registry. Keep these pages aligned with the repository [entity catalog](https://github.com/tharak/lord-of-towers/blob/main/docs/entity-catalog.md), [gathering contracts](https://github.com/tharak/lord-of-towers/blob/main/docs/gathering-contracts.md) and [energy contracts](https://github.com/tharak/lord-of-towers/blob/main/docs/energy-contracts.md).
