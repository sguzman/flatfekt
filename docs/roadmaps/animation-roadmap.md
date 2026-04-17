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
- [ ] Implement tween component(s) for transforms (pos/rot/scale) with easing
- [ ] Implement tween component(s) for opacity/color where applicable
- [ ] Add a minimal easing set (linear, quad in/out, cubic in/out)
- [ ] Add `tracing` instrumentation for timeline start/stop/apply

### M1 — timeline events from TOML
- [ ] Implement timeline event loader and validator (time-ordered, non-negative)
- [ ] Implement event types: apply patch, start tween, stop tween, scene transition (optional)
- [ ] Add deterministic playback mode (fixed dt) behind config knob

### M2 — sequencing + composition
- [ ] Add named tracks and track-level enable/disable
- [ ] Add event grouping (labels) and seek/scrub support (used by UI tooling)
- [ ] Add “relative time” triggers (after event X) with deterministic resolution

### M3 — cinematic polish primitives
- [ ] Add camera pan/zoom presets and transitions
- [ ] Add fade in/out primitives (global or per-entity)

## Grouped tasks

### Determinism and repeatability
- [ ] Ensure tween outcomes are deterministic under fixed dt (tests with golden values)
- [ ] Ensure timeline event ordering is stable for equal timestamps

