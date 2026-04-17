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
- [ ] Define `AssetRef` type (logical id vs path) and TOML representation
- [ ] Implement asset root directory config (`app.assets_dir`) and path safety rules
- [ ] Implement image and font resolution and load hooks (enough for sprites/text)
- [ ] Add `tracing` spans for asset resolution/load/reload

### M1 — caching and dedup
- [ ] Add caching policy (deduplicate by logical id/path)
- [ ] Add asset metadata tracking (size, type, load time)

### M2 — reload behavior
- [ ] Define and implement asset hot reload semantics (with runtime hot reload)
- [ ] Add config knobs for reload debounce and failure policy

### M3 — asset packs / packaging
- [ ] Define pack layout (directory manifest) and implement loading from a pack root
- [ ] Add pack validation tooling (belongs to tooling but implemented here)

