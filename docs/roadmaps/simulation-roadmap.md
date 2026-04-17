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
- [ ] Add sim pause/step/reset actions (wired through `interaction-roadmap.md`)
- [x] Add `tracing` spans around sim tick and system sets

### M1 — simple physics integration (feature-gated)
- [ ] Choose and integrate a maintained 2D physics crate behind a feature flag (document choice)
- [ ] Define schema mapping for rigid bodies/colliders (owned by schema+runtime integration)
- [ ] Implement deterministic stepping integration tests (positions after N ticks)

### M2 — constraints and fields
- [ ] Add force field systems (attract/repel) driven by config
- [ ] Add constraints (bounds, springs) driven by config

### M3 — advanced simulation primitives
- [ ] Add particle system stepping (not rendering; just state evolution)
- [ ] Add cellular automata grid stepping (configurable rules)

## Grouped tasks

### Determinism policy
- [ ] Define which sim features are required-deterministic and enforce via tests
- [x] Add seed routing for stochastic sim systems (no hidden randomness)
