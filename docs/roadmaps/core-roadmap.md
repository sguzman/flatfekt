# Core roadmap

## Purpose
Define the project constitution: workspace boundaries, dependency rules, determinism stance, logging/error standards, config control-pane policy, and “done” criteria used by all other roadmaps.

## Non-goals
- Feature work that belongs to other axes (schema/runtime/rendering/etc.).

## Dependencies
- None (this is the root).

## Public surface
- Workspace crate layout and dependency graph rules.
- Project-wide configuration loading conventions (control pane TOML).
- Project-wide observability conventions (`tracing`).

## Dependency rules (initial)
- [x] Publish and enforce crate dependency DAG (no cycles; `*_runtime` depends on `*_schema`, never vice versa)
  - [x] Instantiate a Cargo workspace with `crates/` and `apps/` members
  - [x] Add initial engine crates: config + schema + runtime
  - [x] Add initial runner app crate
  - [x] Add an explicit “allowed dependencies” document/table and keep it current
  - [x] Add an automated workspace dependency rule check (tests)
- [x] Define “engine crates” vs “apps/examples” (apps may depend on engine crates; engine crates must not depend on apps)
- [x] Define which crates are allowed to touch Bevy types directly (prefer keeping “schema” crates Bevy-free)

## Milestones

### M0 — repository contract exists
- [x] Create root `docs/architecture.md` with subsystem map and boundaries
- [x] Define a single config entrypoint (`flatfekt.toml`) and lookup rules (cwd, env override)
  - [x] Support explicit config path override via `FLATFEKT_CONFIG`
  - [x] Support config file in repository working directory (e.g., `./flatfekt.toml`)
  - [x] Support default config directory `.config/flatfekt/` (prefer `.config/flatfekt/flatfekt.toml`)
- [x] Define error-handling rules (use `thiserror` + `anyhow` boundaries; never `unwrap()` in engine paths)
- [x] Define `tracing` policy (event fields, span boundaries, per-subsystem targets)

### M1 — conventions enforced in code
- [x] Add `deny`/`warn` lints in `Cargo.toml` or `.cargo/config.toml` (minimal, practical)
- [x] Add `cargo fmt` + `cargo clippy` + `cargo test` command set in root `README.md`
- [x] Add a small “engine bootstrap” app demonstrating config load + tracing init + scene load

### M2 — determinism and stability
- [x] Define determinism tiering (deterministic sim mode vs “best-effort realtime” mode)
- [x] Add deterministic RNG policy (seed routing; no hidden entropy sources)
- [x] Add config schema versioning policy (semantics, not semver)

### M3 — long-term hygiene
- [x] Add compatibility policy for scene format (backward-compat windows, migration tooling hooks)
- [x] Define performance budget instrumentation conventions (frame time, sim tick, asset load)

## Config control-pane policy
- [x] Define “must be config” vs “may be hardcoded” rules in `docs/architecture.md`
- [x] Add `flatfekt.toml` sample with comments for all implemented knobs
- [x] Ensure knobs are centralized: no gameplay/scene policy magic numbers outside config without explicit rationale

## Operational directories and run modes
- [x] Add `app.mode` config (`dev`|`prod`) and wire it to operational behavior
- [x] In `dev` mode, write run-scoped timestamped log files under `.cache/flatfekt/logs/` in addition to terminal output
- [x] Standardize cache directory layout under `.cache/flatfekt/` (central cache root)
- [x] Create per-scene cache directories under `.cache/flatfekt/scene/<scene>/` for derived artifacts

## Open design questions
- [x] Decide whether the scene format is one TOML file or a root + includes (directory packs)
- [x] Decide whether Bevy schedule is authoritative, or engine defines its own schedule sets
