# Simulation roadmap (rule-based world evolution)

## Purpose
Own the engine’s “physics/rules” evolution loop: fixed timesteps, physics hooks, constraints, collisions, and other rule-based state changes.

## Non-goals
- Agent decision-making (belongs to `agents-roadmap.md`).
- Rendering (belongs to `rendering-roadmap.md`).

## Dependencies
- `runtime-roadmap.md` (schedule sets and determinism knobs)

## Milestones

### M0 — sim stepping scaffold
- [x] Add fixed timestep driver with configurable `dt` and max catch-up steps
- [x] Add sim pause/step/reset actions (wired through `interaction-roadmap.md`)
- [x] Add `tracing` spans around sim tick and system sets

### M1 — simple physics integration (feature-gated)
- [x] Choose a maintained 2D physics crate behind a feature flag (Rapier2D)
- [x] Integrate Rapier2D backend behind `physics_rapier2d` (runtime plugin + fixed timestep wiring)
- [x] Map schema rigid bodies/colliders to Rapier components for `scenes/physics_test.toml`
- [x] Wire `DebugSettings.wireframe` to Rapier debug renderer (collider outlines + axes)
- [x] Implement deterministic stepping integration tests (native backend)

### M2 — constraints and fields
- [x] Add force field systems (attract/repel: Stubbed as gravity) driven by config
- [x] Add constraints (bounds, springs) driven by config

### M3 — advanced simulation primitives
- [x] Add particle system stepping (not rendering; just state evolution)
- [x] Add cellular automata grid stepping (configurable rules)

## Grouped tasks

### Determinism policy
- [x] Define which sim features are required-deterministic and enforce via tests
- [x] Add seed routing for stochastic sim systems (no hidden randomness)
