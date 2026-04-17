# Tranche 027 — Text + Assets

## Roadmap Items

### Text
- [x] Add `TextSpan` struct to schema
- [x] Add `RichText` spec using spans in entity spec
- [x] Add font fallback chain config in schema + config
- [x] Add per-letter animation driver (wave, jitter, fade-in) -- Logical component + basic displacement
- [x] Add typewriter reveal system driven by timeline events -- Implemented in runtime

### Assets
- [x] Add image decode policy config (max_width, max_height, max_bytes, fail_fast)
- [x] Add SVG config stub (feature-gated) in schema
- [x] Add video config stub (feature-gated) in schema
- [x] Add audio config stub (feature-gated) in schema
- [x] Add pack layout schema (`AssetPackSpec`)
- [x] Add pack loading stub in assets crate

## Verification Results

- `cargo check -p flatfekt-schema` passes.
- `cargo check -p flatfekt-runtime` passes.
- `cargo check -p flatfekt-assets` passes.
- `update_typewriter` system integrated into simulation tick.
- `spawn_text` handles `TextSpec` with `Option<value>` and joined spans.
