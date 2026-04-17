# Architecture

## Mission
Build a **2D Bevy scene runtime driven by TOML**, suitable for simulations, games, and scripted visuals (timelines/transitions).

## Layers (dependency direction: top -> bottom)

### Apps
Binary crates that configure and run the engine.

### Engine runtime
Orchestrates lifecycle (load/reset/reload/transition), applies patches, and schedules simulation/timeline.

### Schema
Typed scene format and validation. No Bevy dependency.

### Config (control pane)
Typed project configuration that selects scene entrypoints, asset roots, logging policy, and feature flags.

## Crate layout

- `crates/flatfekt-config`: control-pane configuration loading and validation (no Bevy)
- `crates/flatfekt-schema`: scene TOML schema types + validation (no Bevy)
- `crates/flatfekt-runtime`: runtime orchestration APIs (Bevy integration will live here)
- `apps/flatfekt-viewer`: reference runner app (loads config + scene and starts the runtime)

## Observability
- All subsystem boundaries emit structured events/spans via `tracing`.
- Logging level/filter are controlled via config and/or environment overrides.

## Policy: config + scenes are TOML-first
Scenes and scene-internal state are controlled via TOML. Project behavior, policy, feature flags, and tunables are centralized in `flatfekt.toml` (control pane).

