# Tranche 020 (2026-04-16) — asset IDs, caching, and deterministic ordering (10 items)

Selected roadmap items (exactly 10):

- [x] Schema (M1): Add “asset reference” indirection (logical IDs mapped to paths via config)
- [x] Schema (Format governance): Add stable ordering rules for deterministic serialization (if exporting)
- [x] Assets (M1): Add caching policy (deduplicate by logical id/path)
- [x] Assets (M1): Add asset metadata tracking (size, type, load time)
- [x] Rendering: Define stable sorting key for renderables (layer, z, entity_id tie-break)
- [x] Rendering: Add tests verifying ordering is deterministic given the same scene input
- [x] Testing (M0): Add unit tests for config parsing/validation
- [x] Testing (M1): Add golden scene fixtures (`tests/fixtures/scenes/`) and validate them in tests
- [x] Testing (M2): Add deterministic render ordering tests (sorting keys)
- [x] Runtime (M3): Add deterministic ordering guarantees where required (stable entity spawn order)
