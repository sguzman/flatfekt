# Testing roadmap (verification + determinism)

## Purpose
Prove correctness and stability: unit tests, schema compatibility tests, determinism tests, golden scene tests, and performance benchmarks.

## Non-goals
- Manual QA tasks (explicitly prohibited for checkbox items).

## Dependencies
- All axes (this is cross-cutting).

## Milestones

### M0 — basic unit coverage
- [x] Add unit tests for config parsing/validation
- [x] Add unit tests for scene parsing/validation
- [x] Add smoke test that instantiates a minimal scene into a Bevy `App` (headless if possible)

### M1 — golden fixtures
- [x] Add golden scene fixtures (`tests/fixtures/scenes/`) and validate them in tests
- [ ] Add golden patch fixtures (`tests/fixtures/patches/`) and validate apply semantics

### M2 — determinism suite
- [x] Add deterministic sim tick tests (state after N ticks equals expected)
- [x] Add deterministic timeline tests (values at timestamps equal expected)
- [x] Add deterministic render ordering tests (sorting keys)

### M3 — benchmarks (verifiable)
- [ ] Add benchmarks for scene load/instantiate time (criterion or built-in harness)
- [ ] Add benchmarks for hot reload apply time and patch apply time
