# Text roadmap (typography + per-letter effects)

## Purpose
Make text a first-class visual object: font loading, fallback, styled spans, layout blocks, per-letter effects, and typographic transitions.

## Non-goals
- General UI layout/panels (UI axis).

## Dependencies
- `rendering-roadmap.md` (render pipeline)
- `assets-roadmap.md` (font refs and loading)
- `animation-roadmap.md` (time-based changes)

## Milestones

### M0 — basic text in scenes
- [ ] Support text entities in scene TOML (string, font ref, size, color, anchor)
- [ ] Support multiline and alignment options
- [ ] Add tests that load scenes with text and validate component instantiation

### M1 — styled spans
- [ ] Add rich text spec (spans with per-span style: color, weight, italics if available)
- [ ] Add font fallback chain config and validate it

### M2 — per-letter effects (first set)
- [ ] Add per-letter animation driver (wave, jitter, fade-in) with deterministic mode
- [ ] Add timed reveal/caption primitives (typewriter effect) driven by timeline events

### M3 — shader-driven text effects (optional)
- [ ] Add optional shader effects pipeline for text (feature-gated)
- [ ] Add effect parameter schema in TOML with validation

