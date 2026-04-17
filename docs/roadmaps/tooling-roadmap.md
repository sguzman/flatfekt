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
- [ ] Add a CLI subcommand: `validate <scene.toml>` (exit non-zero on errors)
- [ ] Add a CLI subcommand: `run <scene.toml>` (overrides config scene path)
- [ ] Add `tracing` output controls via CLI flags (level/filter)

### M1 — schema and formatting helpers
- [ ] Add a TOML formatter/linter for scene files (deterministic output)
- [ ] Add a “print resolved scene” command (after defaults/templates applied)

### M2 — migration tooling
- [ ] Add a scene schema migrator command (vN -> vN+1)
- [ ] Add a patch/delta diff tool (scene A -> scene B)

### M3 — content generators
- [ ] Add “new scene” template generator (minimal working example)
- [ ] Add demo generator for text effects / timeline examples

