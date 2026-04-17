# Agents roadmap (intentional actors)

## Purpose
Own agent state and decision-making frameworks (FSM/BT/utility) as *pluggable* systems operating over simulation/world state.

## Non-goals
- Raw physics/collisions (simulation axis).
- UI panels for inspection (UI axis).

## Dependencies
- `simulation-roadmap.md` (world evolution loop hooks)
- `schema-roadmap.md` (agent parameter schemas)
- `runtime-roadmap.md` (entity ID mapping)

## Milestones

### M0 — agent state + tick hook
- [ ] Define `Agent` component schema (id, parameters, state blob)
- [ ] Add agent tick system set scheduled after sim step
- [ ] Add structured `tracing` for agent tick timing and decisions (sampling configurable)

### M1 — simple decision framework
- [ ] Implement finite state machine (FSM) with data-driven transitions
- [ ] Implement perception helpers (nearby entities, tags, distance queries)
- [ ] Add tests for deterministic decision outcomes given fixed inputs

### M2 — behavior trees / utility AI (one first)
- [ ] Implement one advanced controller (BT or utility AI) with TOML-configured graphs
- [ ] Add debug trace output that can be consumed by UI tooling (structured events)

### M3 — multi-agent coordination
- [ ] Add coordination primitives (shared resources, signals) with deterministic semantics
- [ ] Add flocking/social rule systems behind feature flags

## Grouped tasks

### Data-driven parameters
- [ ] Define agent parameter typing (numbers, bools, vectors) with validation
- [ ] Add parameter defaulting and overrides (scene defaults -> entity -> agent)

