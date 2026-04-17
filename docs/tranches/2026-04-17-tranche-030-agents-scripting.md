# Tranche 030 — Agents + Scripting

## Roadmap Items
- [x] Add `AgentSpec` and `ScriptHookSpec` to schema.
- [x] Implement agent tick system.
- [x] Implement simple FSM framework for agents.
- [x] Implement hook registry for scripting.
- [x] Update `agents-roadmap.md` and `scripting-roadmap.md`.

## Changes
- Added `AgentSpec` and `ScriptHookSpec` to `flatfekt-schema`.
- Implemented `agents.rs` in `flatfekt-runtime`.
- Registered agents module and systems in `lib.rs`.
