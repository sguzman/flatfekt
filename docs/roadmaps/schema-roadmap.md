# Schema roadmap (TOML scene format)

## Purpose
Define the TOML schema that describes scenes as data: entities, components, assets references, defaults/templates, and validation. This axis owns the *format* and its versioning/migrations.

## Non-goals
- Runtime implementation details (belongs to `runtime-roadmap.md`).
- Rendering quality/performance tuning (belongs to `rendering-roadmap.md`).

## Dependencies
- `core-roadmap.md` (format versioning policy, config conventions).

## Public surface
- `scene.toml` (and related pack layout if applicable).
- Schema version identifier and migration strategy.
- Validation errors (stable, actionable messages).

## Milestones

### M0 — minimal scene v0.1
- [x] Define top-level `scene` table layout and required fields
- [x] Define stable `entity_id` type (string) and uniqueness rules
- [x] Define transform representation (2D position/rotation/scale; z-order if needed)
- [x] Define color representation (sRGB triples + alpha)
- [x] Define sprite spec (image ref + size + anchor)
- [x] Define text spec (string + font ref + size + alignment/anchor)
- [x] Define camera spec (2D camera params + clear color)
- [x] Define schema validation rules and error messages (missing fields, bad refs, invalid ranges)

### M1 — defaults, templates, composition
- [x] Add `defaults` table for common settings (fonts, colors, anchor defaults)
- [ ] Add “prefab/template” mechanism (named component bundles) and `extends` semantics
- [x] Add entity tags/groups for selection and bulk operations
- [x] Add “asset reference” indirection (logical IDs mapped to paths via config)

### M2 — deltas/patches and time
- [ ] Define patch format for entity add/remove/update (stable operations)
- [ ] Define patch addressing (by `entity_id`; optional selectors by tag)
- [ ] Define patch validation (referential integrity, type safety)
- [ ] Define timeline event spec (time, action, target, payload)

## Scene playback metadata (video-like scenes)
- [x] Add scene-level duration metadata in TOML (explicit `duration_secs` or equivalent)
- [x] Add scene-level playback policy fields (allow_user_input, allow_scrub/rewind, loop mode)
- [x] Add scene-level introspection toggle (enable/disable inspection features per scene)

### M3 — expressiveness and safety
- [ ] Add conditional activation fields (feature flags, platform flags) with deterministic semantics
- [ ] Add strict schema versioning with migration stubs (format evolution without breaking consumers)

## Grouped tasks

### Format governance
- [x] Add `schema_version` field and document semantics
- [x] Add “unknown fields” policy (reject vs allow-with-warning) and implement it
- [x] Add stable ordering rules for deterministic serialization (if exporting)

### Validation ergonomics
- [x] Add error paths (e.g., `scene.entities[3].sprite.image`) to all validation failures
- [ ] Add “did you mean” suggestions for unknown IDs (optional but useful)

### Documentation artifacts (verifiable)
- [ ] Add a machine-checked schema doc generator (e.g., emit Markdown from Rust types) behind `tooling`
- [x] Add example TOML fixtures used by tests (kept under `tests/fixtures/`)
