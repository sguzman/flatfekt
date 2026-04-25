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
- [x] Add frame export job runner (deterministic fixed-dt stepping + per-frame capture)
- [x] Add `flatfekt export-frames <scene.toml|bake_dir|bake.json>` (auto-bake when given `scene.toml`)
- [x] Add `[export.frames]` control-pane knobs (output_root, width/height, fps/duration defaults, window_visible, overwrite)
- [x] Add export-frames manifest output (fps/duration/frame_count + source metadata)
- [x] Add tests for export path building + config validation (no-GPU)

### M1b — video export (optional)
- [x] Add MP4 encoding pipeline via `ffmpeg` CLI (fail fast if missing/unusable)
- [x] Add `flatfekt export-mp4 <scene.toml|bake_dir|bake.json>` (exports frames then encodes mp4)
- [x] Add `[export.video]` control-pane knobs (ffmpeg path, codec, preset, crf/bitrate, pix_fmt, keep_frames)
- [x] Add tests for ffmpeg argument building + validation (no-GPU)
- [x] Add feature-gated GPU export integration smoke test (bake + export 0.1s to temp dir)
- [x] Improve export CLI output (paths + summary) and support `--out <dir>` for `export-mp4`
- [x] Reduce per-frame export logging (timeline seek) to debug level

### M2 — replay
- [x] Add input/timeline event recording format (timestamped)
- [x] Add replay runner that reproduces a run deterministically (when configured)
- [x] Add simulation baking (bake command, trajectory export, playback interpolation)
- [x] Fix `bake` to run headless (no window), advance simulation time, and write output for `scenes/physics_test.toml`
- [x] Promote bake output to first-class artifact directory under `.cache/flatfekt/scene/<scene>/bakes/<scene_xxhash>/run-.../` (includes `bake.json`, `scene_playback.toml`, and packaged `assets/`)
- [x] Upgrade `bake.json` to v0.2 (meta + playback timing + asset manifest + keyframes: transform + text value + sprite color)
- [x] Add `play-bake` command that runs baked playback without simulation/timeline execution
- [x] Resolve `scene.baked` relative to the scene file location (so `scene_playback.toml` can use `bake.json`)

### M3 — packaged scenes
- [x] Define package format (scene + assets manifest)
- [x] Add pack builder tool that validates and emits a distributable directory
