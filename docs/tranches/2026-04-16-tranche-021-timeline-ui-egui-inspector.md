# Tranche 021 (2026-04-16) — scene playback metadata + timeline clock + egui/inspector (10 items)

Selected roadmap items (exactly 10):

- [x] Schema (Scene playback): Add scene-level duration metadata in TOML (`duration_secs`)
- [x] Schema (Scene playback): Add scene-level playback policy fields (allow_user_input, allow_scrub/rewind, loop mode)
- [x] Schema (Scene playback): Add scene-level introspection toggle (enable/disable inspection features per scene)
- [x] Runtime (M3): Add config knobs under `runtime.timeline` (enabled, fixed_dt_secs, max_catchup_steps)
- [x] Runtime (M3): Implement a `TimelineClock` resource (playing/paused, current time, step)
- [x] Runtime (M3): Wire timeline driver to `SimTick` set (advance by fixed dt when enabled)
- [x] Runtime (M3): Enforce scene duration/end-of-scene behavior (stop/loop) when duration is present
- [x] UI (M0): Add config flags under `features` (`ui_egui`, `inspector_egui`)
- [x] UI (M0): Add `bevy_egui` integration when `features.ui_egui` is enabled
- [x] UI (Introspection): Add `bevy-inspector-egui` integration when `features.inspector_egui` is enabled, gated by scene policy
