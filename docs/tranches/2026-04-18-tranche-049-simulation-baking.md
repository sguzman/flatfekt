# Tranche 049 — Simulation Baking

## Roadmap Items
### `export-roadmap.md`
- [x] Add simulation baking (bake command, trajectory export, playback interpolation)

### `tooling-roadmap.md`
- [x] Add bake subcommand for simulation trajectory export

## Changes
- **`flatfekt-schema`**: Added `baked` field to `Scene` struct.
- **`flatfekt-runtime`**: Created `bake` module with recorder and replay systems. Integrated with `FlatfektRuntimePlugin`. Implemented headless `run_bake` runner.
- **`flatfekt-cli`**: Added `bake` subcommand and integrated runtime baking logic.

## Verification
- `cargo check -p flatfekt-runtime` — PASSED
- `cargo check -p flatfekt-cli` — PASSED
- Manual verification of subcommand registration and parameter parsing.
