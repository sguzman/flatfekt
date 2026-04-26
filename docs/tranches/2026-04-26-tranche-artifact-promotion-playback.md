# Tranche: Bake Artifact Promotion + Playback (2026-04-26)

Roadmap items in this tranche (only):

- [x] `docs/roadmaps/export-roadmap.md` — Promote bake output to first-class artifact directory under `.cache/flatfekt/scene/<scene>/bakes/<scene_xxhash>/run-.../` (includes `bake.json`, `scene_playback.toml`, and packaged `assets/`)
- [x] `docs/roadmaps/export-roadmap.md` — Upgrade `bake.json` to v0.2 (meta + playback timing + asset manifest + keyframes: transform + text value + sprite color)
- [x] `docs/roadmaps/export-roadmap.md` — Add `play-bake` command that runs baked playback without simulation/timeline execution
- [x] `docs/roadmaps/export-roadmap.md` — Resolve `scene.baked` relative to the scene file location (so `scene_playback.toml` can use `bake.json`)
- [x] `docs/roadmaps/core-roadmap.md` — In `dev` mode, write run-scoped timestamped log files under `.cache/flatfekt/logs/` in addition to terminal output
