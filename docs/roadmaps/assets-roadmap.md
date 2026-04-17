# Assets roadmap (resource resolution + management)

## Purpose
Own how assets are referenced from TOML, resolved to paths, loaded/cached, reloaded, and packaged. This includes images, fonts, audio (future), and asset packs.

## Non-goals
- Rendering usage details (rendering axis).
- Export/recording (export axis).

## Dependencies
- `core-roadmap.md` (config policy, errors, tracing)

## Milestones

### M0 — asset reference model v0.1
- [x] Define `AssetRef` type (logical id vs path) and TOML representation
- [x] Implement asset root directory config (`app.assets_dir`) and path safety rules
- [x] Implement image and font resolution and load hooks (enough for sprites/text)
- [x] Add `tracing` spans for asset resolution/load/reload

### M1 — caching and dedup
- [x] Add caching policy (deduplicate by logical id/path)
- [x] Add asset metadata tracking (size, type, load time)

### M2 — reload behavior
- [x] Define and implement asset hot reload semantics (with runtime hot reload)
- [x] Add config knobs for reload debounce and failure policy

### M3 — asset packs / packaging
- [ ] Define pack layout (directory manifest) and implement loading from a pack root
- [ ] Add pack validation tooling (belongs to tooling but implemented here)

## Format support (grouped by type)

### Raster images
- [ ] Support common raster image formats for sprites (png, jpg/jpeg, webp; others as feasible)
- [ ] Add config-driven image decode policy (max dimensions / max bytes; fail-fast vs warn)

### Vector images (SVG)
- [ ] Add SVG asset support via rasterization pipeline (feature-gated)
- [ ] Add SVG rasterization config (target pixels-per-unit, max size, cache keying)

### Video
- [ ] Add video asset abstraction (decode backend feature-gated)
- [ ] Add “video as texture” playback pipeline (decode -> upload -> sprite)
- [ ] Add video policy knobs (max resolution/fps, buffering strategy, audio handling stance)

### Shaders (WGSL)
- [x] Support WGSL shader assets (materials/effects) referenced from TOML
- [x] Add shader compilation/validation error reporting with actionable paths
- [x] Add shader hot-reload integration (ties into runtime hot reload)

### Audio (future)
- [ ] Add audio asset reference model (formats + policy knobs) behind a feature flag
