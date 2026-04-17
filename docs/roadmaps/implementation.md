# flatfekt — implementation roadmap

Goal: a 2D Bevy scene runtime driven by TOML, suitable for simulations, games, and scripted visuals (timelines/transitions), with strong module boundaries, `tracing` instrumentation, and a centralized TOML control-pane config.

Reference projects:
- `tmp/atmos` (architecture reference; 3D-focused, treat as patterns to reuse, not behavior to match)
- `tmp/bevy-flatr` (2D Bevy reference; useful for early rendering/input/world scaffolding)

## Tranche 1 — workspace + config-driven 2D scene bootstrap

- [ ] Convert repo to a Cargo workspace with an engine crate and a runner binary
- [ ] Add core dependencies (latest): `bevy`, `tracing`, `tracing-subscriber`, `thiserror`, `serde`, `toml`
- [ ] Add typed `flatfekt.toml` control-pane config loader with validation and defaults
- [ ] Initialize structured `tracing` subscriber and bridge Bevy logging as needed
- [ ] Define minimal TOML scene schema (2D entities + transforms + visuals + text)
- [ ] Implement scene loader that instantiates a scene into a Bevy `World`
- [ ] Add feature flags for optional subsystems (e.g., `gamepad`, `ui_overlay`)
- [ ] Add `README.md` with `cargo run`, `cargo fmt`, `cargo clippy` commands
- [ ] Add minimal tests for config + scene parsing/validation
- [ ] Ensure `cargo build` succeeds (debug profile)

## Tranche 2 — 2D scene capabilities (first useful subset)

- [ ] Implement `WorldPlugin` that applies window/camera/background/UI overlay from config
- [ ] Implement sprite and shape spawning from scene TOML (colors, sizes, z-order)
- [ ] Implement text spawning from scene TOML (font size, alignment, anchors)
- [ ] Implement input bindings in TOML for basic actions (quit, reset, toggle overlay)
- [ ] Add scene hot-reload support (file watch + reload on change) behind a feature flag
- [ ] Add examples under `examples/` that demonstrate scene composition from TOML
- [ ] Ensure reference parity where appropriate by reusing patterns from `tmp/bevy-flatr`

## Tranche 3 — simulation + temporal control primitives

- [ ] Add timeline/tween primitives for time-based scene changes (configurable)
- [ ] Add “delta”/patch format for scene updates over time (apply/remove/update entities)
- [ ] Add simulation step driver (fixed timestep) with hooks for agent systems
- [ ] Expand `tracing` spans at boundaries (startup, load, reload, tick, apply-delta)
- [ ] Improve error reporting for config/scene failures (actionable messages)
