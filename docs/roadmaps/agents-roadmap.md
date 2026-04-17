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
- [x] Define `Agent` component schema (id, parameters, state blob)
- [x] Add agent tick system set scheduled after sim step
- [x] Add structured `tracing` for agent tick timing and decisions (sampling configurable)

### M1 — simple decision framework
- [x] Implement finite state machine (FSM) with data-driven transitions
- [x] Implement perception helpers (nearby entities, tags, distance queries)
- [x] Add tests for deterministic decision outcomes given fixed inputs

### M2 — behavior trees / utility AI (one first)
- [x] Implement one advanced controller (BT or utility AI) with TOML-configured graphs
- [x] Add debug trace output that can be consumed by UI tooling (structured events)

### M3 — multi-agent coordination
- [x] Add coordination primitives (shared resources, signals) with deterministic semantics
- [x] Add flocking/social rule systems behind feature flags

## Grouped tasks

### Data-driven parameters
- [x] Define agent parameter typing (numbers, bools, vectors) with validation
- [x] Add parameter defaulting and overrides (scene defaults -> entity -> agent)

