# Tranche 019 (2026-04-16) — rendering/schema fixtures and basics (10 items)

Selected roadmap items (exactly 10):

- [x] Schema (Format governance): Add `schema_version` field and document semantics
- [x] Schema (Format governance): Add “unknown fields” policy (reject vs allow-with-warning) and implement it
- [x] Schema (M1): Add `defaults` table for common settings (fonts, colors, anchor defaults)
- [x] Schema (M1): Add entity tags/groups for selection and bulk operations
- [x] Schema (Docs artifacts): Add example TOML fixtures used by tests (kept under `tests/fixtures/`)
- [x] Rendering (M0): Spawn basic shapes (rect, circle, polygon) with color and size
- [x] Rendering (M1): Define coordinate system policy (pixels vs world units) and implement it consistently
- [x] Rendering (M1): Add viewport scaling modes (fit, fill, pixel-perfect) configurable
- [x] Rendering (M1): Add anchor/origin semantics for sprites/shapes/text and test them with fixtures
- [x] Testing (M0): Add unit tests for scene parsing/validation
