# Collaboration rules

Use Bevy ECS for gameplay and UI. Shared contracts are in src/model.rs; coordinate changes with the lead agent. Keep simulation independent from mesh and material handles. Use only Bevy primitive meshes. Add all new application entity kinds and lifecycle rules to docs/entity-catalog.md.

Validation is compilation only: native cargo check and a release WebAssembly build in GitHub Actions. Do not add or run automated tests, browser suites, extended simulations, or runtime smoke checks. Investigate runtime issues when the user reports them. Local compilation is allowed when explicitly requested; formatting is allowed. Each agent owns its assigned files and must commit changes on its assigned branch. Never merge another agent's branch or alter shared interfaces without coordinating.
