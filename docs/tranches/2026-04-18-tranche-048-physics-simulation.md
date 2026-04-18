# Tranche 048 — Physics Simulation Support

Implementing basic physics integration for falling and bouncing entities.

## Roadmap items
- [simulation-roadmap.md]
  - M1: Choice and integration of native physics stubs (expanding)
  - M2: Add constraints (bounds) driven by config
- [runtime-roadmap.md]
  - M0: Implement instantiation of physics components

## Changes
- `flatfekt-runtime/src/simulation.rs`: Add `Collider`, `SimRegionRes`, and bounds collision logic.
- `flatfekt-runtime/src/lib.rs`: Hook up physics instantiation.
- `scenes/physics_test.toml`: Add test scene.
