# flatfekt — implementation roadmap

Goal (current best inference): build a small Bevy-based “flatland” prototype, migrated from `tmp/bevy-flatr`, with structured `tracing`-based logging and a centralized TOML control-pane config.

## Tranche 1 — foundation + migration skeleton

- [ ] Add crate structure: `lib` + `bin` (`src/lib.rs`, `src/main.rs`) aligned with `tmp/bevy-flatr`
- [ ] Add core dependencies: `bevy`, `tracing`, `tracing-subscriber`, `thiserror`, `serde`
- [ ] Add `flatfekt.toml` config loader module (typed config + validation)
- [ ] Bridge Bevy/log to `tracing` and set up structured subscriber initialization
- [ ] Port plugin module skeletons from `tmp/bevy-flatr`: `world`, `player`, `citizens`, `dialogue`, `input`
- [ ] Move narrative script lines into config (no hardcoded narrative in `main`)
- [ ] Replace gameplay constants with config-driven parameters (no scattered magic numbers)
- [ ] Add `cargo fmt` + `cargo clippy` CI-friendly commands documented in `README.md`
- [ ] Add minimal smoke test(s) for config parsing/validation
- [ ] Ensure `cargo build` succeeds (debug profile)

## Tranche 2 — feature parity with reference

- [ ] Implement `WorldPlugin` port (camera, background, UI text) using config values
- [ ] Implement `CitizensPlugin` port (spawn + wander + tint) using config values and seed
- [ ] Implement `PlayerPlugin` port (keyboard + gamepad) using config values
- [ ] Implement `DialoguePlugin` port (proximity whispers + lifetime) using config values
- [ ] Implement `InputPlugin` port (escape-to-quit) with tracing instrumentation
- [ ] Verify runtime parity against `tmp/bevy-flatr` where applicable (structure + behavior)

## Tranche 3 — operational hardening (still small)

- [ ] Add structured spans/events at major boundaries (startup, config load, plugin init)
- [ ] Add error context for config and startup failures (clear, actionable messages)
- [ ] Add feature flags for optional subsystems (e.g., `gamepad`, `ui_help_text`)
- [ ] Add release profile tuning notes (kept minimal, only if needed)

