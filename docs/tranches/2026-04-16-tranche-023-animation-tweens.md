# Tranche 023 (2026-04-16) — Animation Primitives and Tweens (10 items)

Selected roadmap items (exactly 10):

- [x] Animation (M0): Implement tween component(s) for transforms (pos/rot/scale) with easing
- [x] Animation (M0): Implement tween component(s) for opacity/color where applicable
- [x] Animation (M0): Add a minimal easing set (linear, quad in/out, cubic in/out)
- [x] Animation (M0): Add `tracing` instrumentation for timeline start/stop/apply
- [x] Animation: Add rewind semantics definition (what “rewind” means for tweens/patches) and implement it
- [x] Animation: Add seek-to-time API (jump to timestamp deterministically) with tests
- [x] Animation (M3): Add camera pan/zoom presets and transitions
- [x] Animation (M3): Add fade in/out primitives (global or per-entity)
- [x] Animation (Grouped): Ensure tween outcomes are deterministic under fixed dt (tests with golden values)
- [x] Animation (Grouped): Ensure timeline event ordering is stable for equal timestamps
