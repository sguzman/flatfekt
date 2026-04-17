# Runtime roadmap (scene instantiation + lifecycle)

## Purpose
Turn schema into a running Bevy world: load, instantiate, reset, reload, transition between scenes, apply patches, schedule systems, and manage lifecycles deterministically where desired.

## Non-goals
- Rendering features beyond instantiating components.
- UI tooling (inspector panels).

## Dependencies
- `schema-roadmap.md` (format types + validation)
- `assets-roadmap.md` (asset resolution/load policy)
- `core-roadmap.md` (`tracing` + errors)

## Public surface
- Scene loader API: `load_scene(path) -> SceneSpec`
- Scene instantiator API: `spawn_scene(spec) -> SceneHandle`
- Patch applier API: `apply_patch(scene, patch)`

## Milestones

### M0 — bootstrap runner
- [x] Implement `flatfekt.toml` load + validate at startup (fail fast with clear errors)
- [x] Implement scene TOML load + validate at startup
- [x] Implement instantiation of: camera, sprites, text, basic transforms
- [x] Add structured `tracing` spans around config load, scene load, instantiate

### M1 — lifecycle + hot reload
- [x] Implement “reset scene” (despawn and re-instantiate deterministically)
- [ ] Implement hot reload (file watch + debounce + reload) behind `features.hot_reload`
- [ ] Ensure hot reload surfaces actionable errors without crashing the app loop

### M2 — patches + transitions
- [ ] Implement patch application (add/remove/update entities)
- [ ] Implement scene-to-scene transitions (clear old scene + load new) with configurable strategy
- [ ] Add “scene state snapshot” for deterministic replay (serialize minimal state)

### M3 — scheduling and determinism
- [x] Define engine schedule sets (Load, SimTick, RenderPrep, UI, etc.)
- [x] Add fixed timestep + timeline driver option for sim determinism
  - [x] Add config knobs under `runtime.timeline` (enabled, fixed_dt_secs, max_catchup_steps)
  - [x] Implement a `TimelineClock` resource (playing/paused, current time, step)
  - [x] Wire timeline driver to `SimTick` set (advance by fixed dt when enabled)
  - [x] Enforce scene duration/end-of-scene behavior (stop/loop) when duration is present
- [x] Add deterministic ordering guarantees where required (stable entity spawn order)

## Grouped tasks

### Handles and IDs
- [x] Define runtime entity mapping: `entity_id` -> `Entity`
- [x] Implement lookup helpers with `tracing` instrumentation on failure paths

### Error policy
- [x] Convert loader failures into structured errors with context (file path, field path)
- [ ] Add “warn and continue” policy for non-fatal reload errors (configurable)
