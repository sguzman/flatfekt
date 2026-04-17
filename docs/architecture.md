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

## Bevy dependency boundary
- Crates that may depend on Bevy: runtime/rendering/UI “engine” crates and apps.
- Crates that must remain Bevy-free: schema/config/tooling crates (so they can run in validators/CLI tools without pulling a renderer).

## Toolchain and formatting
- The repository pins its toolchain via `rust-toolchain.toml` (nightly + required components).
- Formatting is defined by `rustfmt.toml` (intentionally aggressive); `cargo fmt` output is the canonical style.

## Determinism stance (initial)
The project supports both:

- **Deterministic mode**: fixed timestep, seeded randomness, deterministic ordering guarantees where specified.
- **Realtime mode**: best-effort framerate; may trade strict determinism for responsiveness.

Determinism is treated as a *configurable policy* rather than an implicit property.

## RNG policy (initial)
- No “hidden” randomness in engine code: stochastic behavior must be driven by an explicit, configurable seed.
- Seed values must come from configuration or explicit runtime inputs, and be recorded in logs in deterministic mode.

## Scene format compatibility policy (initial)
- Scene schema is versioned with a **schema version string** (semantic meaning, not semver).
- Backward compatibility should be preserved within a declared compatibility window.
- When incompatibilities are introduced, migration tooling hooks are required (see tooling roadmap).

## Performance instrumentation conventions (initial)
- All major phases should be observable with `tracing` spans/events:
  - config load/validate
  - scene load/validate
  - instantiate
  - hot reload apply
  - patch apply
  - sim tick and timeline tick (when enabled)

Per-frame spam should be avoided; use spans and sampled events where appropriate.

## Scene packaging decision
Initial decision: **single scene TOML file** as the entrypoint (no includes) for v0.1.
Future work may add a “scene pack” layout (root + includes) once validation/migration tooling exists.

## Scheduling decision
Initial decision: the engine defines its own **schedule sets** layered on top of Bevy (Bevy remains the underlying scheduler, but engine-owned sets control order and boundaries).

## Observability
- All subsystem boundaries emit structured events/spans via `tracing`.
- Logging level/filter are controlled via config and/or environment overrides.

## Policy: config + scenes are TOML-first
Scenes and scene-internal state are controlled via TOML. Project behavior, policy, feature flags, and tunables are centralized in `flatfekt.toml` (control pane).

## Control pane policy

### Must be configuration
These must be represented in the control pane TOML (not hardcoded):

- feature flags and subsystem enable/disable switches
- limits and thresholds (sizes, counts, caps, debounce timers)
- performance/determinism policies (fixed dt, max catch-up steps, seeds)
- file paths and operational directories (config path, cache root, logs, assets root)
- tuning knobs intended to change between runs (movement rates, UI font sizes, time scaling)

### May be hardcoded (with rationale)
Hardcoding is acceptable only when the value is:

- required by a crate API and not meaningfully configurable, or
- a stable internal invariant that should not vary between runs, or
- a temporary bootstrap default that is also documented and can be overridden via config

When a value is intentionally hardcoded, the surrounding code should clearly indicate why it is not in config.
