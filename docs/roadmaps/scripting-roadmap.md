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
- [ ] Define a “hook registry” mapping schema event names -> Rust system implementations
- [ ] Allow timeline events to call registered hooks with typed payloads
- [ ] Add tests for hook dispatch determinism and error reporting

### M1 — expression language (small)
- [ ] Define minimal expression grammar for conditions (comparisons, boolean ops, literals)
- [ ] Implement evaluator with deterministic semantics and resource limits
- [ ] Wire expressions into event triggers/guards

### M2 — embedded scripting (optional, feature-gated)
- [ ] Choose a maintained embedded language (feature-gated; document rationale)
- [ ] Add sandbox/resource limits and clear security documentation

### M3 — plugin API for custom systems
- [ ] Define plugin registration API so external crates can provide hooks/components safely
- [ ] Add versioning and compatibility checks for plugins

