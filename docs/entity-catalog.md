# Entity catalog

The live inspector queries the ECS world; this document records design, not runtime IDs.

| Kind | Components / relationships | Lifecycle | Ticket |
|---|---|---|---|
| Faction | Faction, Name, EntityKind, MatchEntity | Created on match setup; removed on restart | #4 |
| Tower | Tower, Allegiance → Faction, Position, Health, Spawner, InfluenceEmitter, Name, EntityKind, MatchEntity | Five on setup; ownership changes on capture; removed on restart | #4, #6 |
| Minion | Minion, Allegiance → Faction, OriginTower → Tower, Position, Health, Movement, Attack, InfluenceEmitter, Name, EntityKind, MatchEntity | Spawned by tower; dies in combat; removed on restart | #4, #5, #6 |
| Battlefield | EntityKind, Mesh3d, MeshMaterial3d, Transform | Presentation startup; persists across restart | #7, #8 |
| Camera | EntityKind, Camera3d, projection, Transform | Presentation startup; persists across restart | #8 |
| Light | EntityKind, light, Transform | Presentation startup; persists across restart | #8 |
| UI | EntityKind, Name, Node and/or Text/Button | Presentation or inspector startup; dynamic rows may be rebuilt | #3, #8 |

Faction entities remain alive throughout a match, even when they have no towers. Original tower entities remain alive across captures. Gameplay components determine allegiance and physical ranges; visuals are derived. Engine-managed entities may appear under the inspector's unclassified filter.
