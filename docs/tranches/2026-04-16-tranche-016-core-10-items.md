# Tranche 016 (2026-04-16) — core roadmap (10 items)

Selected roadmap items (exactly 10):

- [x] Publish and enforce crate dependency DAG (no cycles; `*_runtime` depends on `*_schema`, never vice versa)
- [x] Define determinism tiering (deterministic sim mode vs “best-effort realtime” mode)
- [x] Add deterministic RNG policy (seed routing; no hidden entropy sources)
- [x] Add config schema versioning policy (semantics, not semver)
- [x] Add compatibility policy for scene format (backward-compat windows, migration tooling hooks)
- [x] Define performance budget instrumentation conventions (frame time, sim tick, asset load)
- [x] Define “must be config” vs “may be hardcoded” rules in `docs/architecture.md`
- [x] Ensure knobs are centralized: no gameplay/scene policy magic numbers outside config without explicit rationale
- [x] Decide whether the scene format is one TOML file or a root + includes (directory packs)
- [x] Decide whether Bevy schedule is authoritative, or engine defines its own schedule sets
