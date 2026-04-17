# Tranche 018 (2026-04-16) — runtime/rendering scaffolding (10 items)

Selected roadmap items (exactly 10):

- [x] Schema (M0): Define transform representation (2D position/rotation/scale; z-order if needed)
- [x] Schema (Validation ergonomics): Add error paths (e.g., `scene.entities[3].sprite.image`) to all validation failures
- [x] Assets (M0): Add `tracing` spans for asset resolution/load/reload
- [x] Runtime (M0): Implement “reset scene” (despawn and re-instantiate deterministically)
- [x] Runtime (M3): Define engine schedule sets (Load, SimTick, RenderPrep, UI, etc.)
- [x] Runtime (Handles and IDs): Define runtime entity mapping: `entity_id` -> `Entity`
- [x] Runtime (Handles and IDs): Implement lookup helpers with `tracing` instrumentation on failure paths
- [x] Runtime (Error policy): Convert loader failures into structured errors with context (file path, field path)
- [x] Rendering (M0): Add background clear color and/or background quad policy
- [x] Rendering (M0): Add camera config: position, zoom, clear color
