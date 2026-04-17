# Scripting roadmap (extensibility)

## Purpose
Allow extending behavior without recompiling the whole engine: expression language, event conditions, and optional embedded scripting—kept safe and deterministic when configured.

## Non-goals
- Core timeline/tween system (animation axis).
- Agent decision frameworks (agents axis).

## Dependencies
- `schema-roadmap.md` (syntax representation)
- `runtime-roadmap.md` (hook points)
- `core-roadmap.md` (security stance)

## Milestones

### M0 — no-scripting hooks
- [x] Define a “hook registry” mapping schema event names -> Rust system implementations
- [x] Allow timeline events to call registered hooks with typed payloads
- [x] Add tests for hook dispatch determinism and error reporting

### M1 — expression language (small)
- [x] Define minimal expression grammar for conditions (comparisons, boolean ops, literals)
- [x] Implement evaluator with deterministic semantics and resource limits
- [x] Wire expressions into event triggers/guards

### M2 — embedded scripting (optional, feature-gated)
- [x] Choose a maintained embedded language (feature-gated; document rationale)
- [x] Add sandbox/resource limits and clear security documentation

### M3 — plugin API for custom systems
- [x] Define plugin registration API so external crates can provide hooks/components safely
- [x] Add versioning and compatibility checks for plugins

