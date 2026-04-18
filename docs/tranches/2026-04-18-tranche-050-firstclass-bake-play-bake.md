# Tranche 050 — First-class bake + play-bake

## Roadmap Items
### `export-roadmap.md`
- [x] Promote bake output to first-class artifact directory under `.cache/flatfekt/scene/<scene>/bakes/<scene_xxhash>/run-.../` (includes `bake.json`, `scene_playback.toml`, and packaged `assets/`)
- [x] Upgrade `bake.json` to v0.2 (meta + playback timing + asset manifest + keyframes: transform + text value + sprite color)
- [x] Add `play-bake` command that runs baked playback without simulation/timeline execution
- [x] Resolve `scene.baked` relative to the scene file location (so `scene_playback.toml` can use `bake.json`)
