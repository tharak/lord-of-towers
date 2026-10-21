# Lord of Towers: shared vocabulary

These pages name the ECS concepts used by the game. They are the team reference for issues, code reviews, and later art changes.

- [Nomenclature](Nomenclature): rawElement, donut, torus, tower, minion, trail, link.
- [Entities](Entities): every application entity kind and its lifetime.
- [Components and Resources](Components-and-Resources): gameplay state, visual markers, and bounded resources.

The live inspector queries the ECS world directly. It is not a second entity registry. The repository's [entity catalog](https://github.com/tharak/lord-of-towers/blob/main/docs/entity-catalog.md) mirrors entity lifecycles. Update both when the model changes.
