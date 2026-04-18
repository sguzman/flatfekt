# Tooling roadmap (developer experience)

## Purpose
Provide tools that make the engine usable: CLI, validators, linters, formatters, diff tools, migration tools, demo generators, and live preview helpers.

## Non-goals
- UI panels (UI axis) unless explicitly a tool UI.

## Dependencies
- `schema-roadmap.md` (validation)
- `assets-roadmap.md` (pack validation)
- `testing-roadmap.md` (golden fixtures usage)

## Milestones

### M0 — validate and run
- [x] Add a CLI subcommand: `validate <scene.toml>` (exit non-zero on errors)
- [x] Add a CLI subcommand: `run <scene.toml>` (overrides config scene path)
- [x] Add `tracing` output controls via CLI flags (level/filter)
- [x] Remove workspace `cargo run` ambiguity (single `flatfekt` bin; set a workspace default run target)

### M1 — schema and formatting helpers
- [x] Add a TOML formatter/linter for scene files (deterministic output)
- [x] Add a “print resolved scene” command (after defaults/templates applied)

### M2 — migration tooling
- [x] Add a scene schema migrator command (vN -> vN+1)
- [x] Add a patch/delta diff tool (scene A -> scene B)

### M3 — content generators
- [x] Add “new scene” template generator (minimal working example)
- [x] Add demo generator for text effects / timeline examples
