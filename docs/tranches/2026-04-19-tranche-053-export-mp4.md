# Tranche 053 — MP4 export (frames + ffmpeg)

Goal: implement first-class MP4 export by exporting a deterministic PNG frame sequence and encoding it via `ffmpeg`.

Source roadmap: `docs/roadmaps/export-roadmap.md`

## Roadmap items executed
- [x] Add frame export job runner (deterministic fixed-dt stepping + per-frame capture)
- [x] Add `flatfekt export-frames <scene.toml|bake_dir|bake.json>` (auto-bake when given `scene.toml`)
- [x] Add `[export.frames]` control-pane knobs (output_root, width/height, fps/duration defaults, window_visible, overwrite)
- [x] Add export-frames manifest output (fps/duration/frame_count + source metadata)
- [x] Add MP4 encoding pipeline via `ffmpeg` CLI (fail fast if missing/unusable)
- [x] Add `flatfekt export-mp4 <scene.toml|bake_dir|bake.json>` (exports frames then encodes mp4)
- [x] Add `[export.video]` control-pane knobs (ffmpeg path, codec, preset, crf/bitrate, pix_fmt, keep_frames)
- [x] Add tests for ffmpeg argument building + validation (no-GPU)
