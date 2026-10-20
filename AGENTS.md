# Collaboration rules

Use Bevy ECS for gameplay and UI. Shared contracts are in src/model.rs; coordinate changes with the lead agent. Keep simulation independent from mesh and material handles. Use only Bevy primitive meshes. Add all new application entity kinds and lifecycle rules to docs/entity-catalog.md.

Testing runs in GitHub Actions, not routinely on the local machine. Write meaningful tests with changes. Do not run local builds/tests unless the lead needs a reproduction of a CI failure. Formatting is allowed. Each agent owns its assigned files and must commit changes on its assigned branch. Never merge another agent's branch or alter shared interfaces without coordinating.
