# Tranche: Aggregate bake + MP4 export (2026-04-25)

Roadmap items in this tranche (only):

- [x] `docs/roadmaps/export-roadmap.md` — Support baking stitched scenes (`scene.sequence[]`) by baking clips and merging into one bake artifact
- [x] `docs/roadmaps/schema-roadmap.md` — Add aggregate scene stitching format (`scene.sequence[]`) referencing other scenes for playback *(refine: allow omitting `entities` field when `scene.sequence[]` is present)*
- [x] `docs/roadmaps/runtime-roadmap.md` — Implement aggregate scene stitching: play a sequence of referenced scenes back-to-back with strict fps/resolution/duration validation *(refine: fix aggregate driver resource borrow so export works)*
