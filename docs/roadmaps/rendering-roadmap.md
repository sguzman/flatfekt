# Rendering roadmap (2D visuals)

## Purpose
Own 2D visual output: camera model, layering/z-order, sprites, shapes, materials, viewport scaling, and optional offscreen rendering.

## Non-goals
- Timeline/tweening logic (belongs to `animation-roadmap.md`).
- Simulation (belongs to `simulation-roadmap.md`).

## Dependencies
- `runtime-roadmap.md` (scene instantiation hooks)
- `assets-roadmap.md` (textures/fonts)

## Milestones

### M0 — basic 2D render primitives from TOML
- [ ] Spawn sprites with explicit z-order/layering semantics
- [ ] Spawn basic shapes (rect, circle, polygon) with color and size
- [ ] Add background clear color and/or background quad policy
- [ ] Add camera config: position, zoom, clear color

### M1 — layout + scaling correctness
- [ ] Define coordinate system policy (pixels vs world units) and implement it consistently
- [ ] Add viewport scaling modes (fit, fill, pixel-perfect) configurable
- [ ] Add anchor/origin semantics for sprites/shapes/text and test them with fixtures

### M2 — materials and effects
- [ ] Add sprite tint/opacity controls
- [ ] Add simple shader/material hooks (optional; feature-gated)
- [ ] Add layered post-processing pipeline hooks (optional; future-ready)

## Effect integration
- [ ] Define TOML-facing effect binding model (per-entity and/or global passes)
- [ ] Add WGSL effect material example (minimal) and ensure it loads from TOML refs

### M3 — offscreen rendering and capture
- [ ] Add render-to-texture support for compositing
- [ ] Add screenshot capture API (used by `export-roadmap.md`)

## Grouped tasks

### Deterministic draw ordering
- [ ] Define stable sorting key for renderables (layer, z, entity_id tie-break)
- [ ] Add tests verifying ordering is deterministic given the same scene input
