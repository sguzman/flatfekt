# Animation roadmap (timeline, tweening, transitions)

## Purpose
Provide time-based change primitives (tweens, keyframes, timelines) for declarative scene choreography: transforms, opacity, colors, camera moves, and scripted sequences.

## Non-goals
- Simulation stepping (belongs to `simulation-roadmap.md`).
- Export/recording (belongs to `export-roadmap.md`).

## Dependencies
- `schema-roadmap.md` (timeline event and patch schema)
- `runtime-roadmap.md` (patch application and scheduling)

## Milestones

### M0 — tween primitives
- [x] Implement tween component(s) for transforms (pos/rot/scale) with easing
- [x] Implement tween component(s) for opacity/color where applicable
- [x] Add a minimal easing set (linear, quad in/out, cubic in/out)
- [x] Add `tracing` instrumentation for timeline start/stop/apply

### M1 — timeline events from TOML
- [x] Implement timeline event loader and validator (time-ordered, non-negative)
- [x] Implement event types: apply patch, start tween, stop tween, scene transition (optional)
- [x] Add deterministic playback mode (fixed dt) behind config knob
- [x] Add detailed `tracing` for timeline dispatch (cursor/clock, per-event action/target/time, parse failures)

### M2 — sequencing + composition
- [x] Add named tracks and track-level enable/disable
- [x] Add event grouping (labels) and seek/scrub support (used by UI tooling)
- [x] Add “relative time” triggers (after event X) with deterministic resolution

## Playback controls
- [x] Add rewind semantics definition (what “rewind” means for tweens/patches) and implement it
- [x] Add seek-to-time API (jump to timestamp deterministically) with tests

### M3 — cinematic polish primitives
- [x] Add camera pan/zoom presets and transitions
- [x] Add fade in/out primitives (global or per-entity)

## Grouped tasks

### Determinism and repeatability
- [x] Ensure tween outcomes are deterministic under fixed dt (tests with golden values)
- [x] Ensure timeline event ordering is stable for equal timestamps
