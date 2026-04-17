# Tranche 010 (2026-04-16) — basic scaffolding (10 roadmap items)

Selected roadmap items (exactly 10):

- [ ] Core (M0): Define a single config entrypoint (`flatfekt.toml`) and lookup rules (cwd, env override)
- [ ] Core (M0): Define error-handling rules (use `thiserror` + `anyhow` boundaries; never `unwrap()` in engine paths)
- [ ] Core (M0): Define `tracing` policy (event fields, span boundaries, per-subsystem targets)
- [ ] Core (DAG): Add an explicit “allowed dependencies” document/table and keep it current
- [ ] Core (Control pane): Add `flatfekt.toml` sample with comments for all implemented knobs
- [ ] Schema (M0): Define top-level `scene` table layout and required fields
- [ ] Schema (M0): Define stable `entity_id` type (string) and uniqueness rules
- [ ] Schema (M0): Define schema validation rules and error messages (missing fields, bad refs, invalid ranges)
- [ ] Runtime (M0): Implement `flatfekt.toml` load + validate at startup (fail fast with clear errors)
- [ ] Runtime (M0): Implement scene TOML load + validate at startup

Completed:

- [x] Core (M0): Define a single config entrypoint (`flatfekt.toml`) and lookup rules (cwd, env override)
- [x] Core (M0): Define error-handling rules (use `thiserror` + `anyhow` boundaries; never `unwrap()` in engine paths)
- [x] Core (M0): Define `tracing` policy (event fields, span boundaries, per-subsystem targets)
- [x] Core (DAG): Add an explicit “allowed dependencies” document/table and keep it current
- [x] Core (Control pane): Add `flatfekt.toml` sample with comments for all implemented knobs
- [x] Schema (M0): Define top-level `scene` table layout and required fields
- [x] Schema (M0): Define stable `entity_id` type (string) and uniqueness rules
- [x] Schema (M0): Define schema validation rules and error messages (missing fields, bad refs, invalid ranges)
- [x] Runtime (M0): Implement `flatfekt.toml` load + validate at startup (fail fast with clear errors)
- [x] Runtime (M0): Implement scene TOML load + validate at startup
