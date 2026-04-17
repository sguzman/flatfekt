# Tranche 024 (2026-04-17) — runtime hot reload hardening + animation timeline events (10 items)

Selected roadmap items (exactly 10):

- [x] Runtime (M1): Implement hot reload (file watch + debounce + reload) behind `features.hot_reload`
- [x] Runtime (M1): Ensure hot reload surfaces actionable errors without crashing the app loop
- [x] Runtime (Error policy): Add “warn and continue” policy for non-fatal reload errors (configurable)
- [x] Runtime (M2): Implement patch application (add/remove/update entities)
- [x] Runtime (M2): Implement scene-to-scene transitions (clear old scene + load new) with configurable strategy
- [x] Runtime (M2): Add “scene state snapshot” for deterministic replay (serialize minimal state)
- [x] Animation: Implement timeline event loader and validator (time-ordered, non-negative)
- [x] Animation: Implement event types: apply patch, start tween, stop tween, scene transition (optional)
- [x] Animation: Add deterministic playback mode (fixed dt) behind config knob
- [x] Animation: Add named tracks and track-level enable/disable
