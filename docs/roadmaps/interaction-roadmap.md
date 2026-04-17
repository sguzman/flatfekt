# Interaction roadmap (input + external control)

## Purpose
Own external control surfaces: keyboard/mouse/gamepad bindings, clickable entities, dev commands, and control events injected into runtime/sim/timeline.

## Non-goals
- UI widgets/layout (UI axis).

## Dependencies
- `runtime-roadmap.md` (scene handles and actions)
- `core-roadmap.md` (`tracing` policy)

## Milestones

### M0 — basic input mapping
- [x] Implement “action map” schema in TOML (actions bound to keys/buttons)
- [x] Provide built-in actions: quit, reset, toggle overlay, pause, step
- [x] Provide built-in actions: play, rewind, fast-forward, seek (when enabled by scene policy)
- [x] Add `tracing` events on action dispatch (action name, source)

### M1 — pointing and selection
- [x] Implement picking (mouse hit test) for 2D entities with selectable components
- [x] Implement click/drag events routed to runtime hooks

### M2 — external control channel (optional)
- [x] Add a CLI command channel (stdin or file-based) behind a feature flag
- [x] Add a local IPC option (e.g., TCP/UDS) behind a feature flag with explicit security stance

### M3 — recording input for replay
- [x] Add input event recording (timestamped) feeding into `export-roadmap.md` replay
