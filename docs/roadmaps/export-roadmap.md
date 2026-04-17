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
- [ ] Implement screenshot capture to file (configurable output dir/name pattern)
- [ ] Add CLI command to trigger screenshot capture (single-shot)

### M1 — frame sequence export (motion graphics)
- [ ] Add “fixed dt render” mode for deterministic frame stepping
- [ ] Export frame sequences (png sequence) for a given duration

### M2 — replay
- [ ] Add input/timeline event recording format (timestamped)
- [ ] Add replay runner that reproduces a run deterministically (when configured)

### M3 — packaged scenes
- [ ] Define package format (scene + assets manifest)
- [ ] Add pack builder tool that validates and emits a distributable directory

