# Export roadmap (artifacts: replay/capture/package)

## Purpose
Own the outputs the engine can produce: save/load, recording/replay, frame export, screenshot capture, and packaged scenes.

## Non-goals
- Tooling UI (tooling axis) except where required to trigger exports.

## Dependencies
- `runtime-roadmap.md` (snapshot/scene state model)
- `rendering-roadmap.md` (screenshot/offscreen render)
- `animation-roadmap.md` (timeline determinism for frame export)

## Milestones

### M0 — screenshot export
- [x] Implement screenshot capture to file (configurable output dir/name pattern)
- [x] Add CLI command to trigger screenshot capture (single-shot)

### M1 — frame sequence export (motion graphics)
- [x] Add “fixed dt render” mode for deterministic frame stepping
- [x] Export frame sequences (png sequence) for a given duration

### M1b — video export (optional)
- [x] Add video encoding pipeline (feature-gated) for exporting to a common container (e.g., mp4)
- [x] Add config knobs for encoder settings (fps, bitrate, pixel format) and fail-fast validation

### M2 — replay
- [x] Add input/timeline event recording format (timestamped)
- [x] Add replay runner that reproduces a run deterministically (when configured)
- [x] Add simulation baking (bake command, trajectory export, playback interpolation)
- [x] Fix `bake` to run headless (no window), advance simulation time, and write output for `scenes/physics_test.toml`

### M3 — packaged scenes
- [x] Define package format (scene + assets manifest)
- [x] Add pack builder tool that validates and emits a distributable directory
