# UI roadmap (overlays + inspector surfaces)

## Purpose
Own interface around the scene: HUD, inspector panels, debug overlays, menus, timeline scrubber, perf panel.

## Non-goals
- Text rendering features themselves (text axis).

## Dependencies
- `interaction-roadmap.md` (input actions)
- `runtime-roadmap.md` (scene/entity lookup)
- `animation-roadmap.md` (timeline controls, if present)

## Milestones

### M0 — basic overlay
- [ ] Add a toggleable help overlay (configurable text)
- [ ] Add a minimal debug overlay showing fps + scene name + tick mode
- [x] Add an embedded control GUI (egui) for basic actions (play/pause/step/reset/toggles) behind a feature flag
  - [x] Add config flags under `features` (`ui_egui`, `inspector_egui`)
  - [x] Add `bevy_egui` integration when `features.ui_egui` is enabled
  - [x] Implement a basic control panel (play/pause/step/reset + time readout)
- [ ] Instrument UI updates with `tracing` only at boundaries (avoid per-frame spam)

### M1 — entity inspector (minimal)
- [ ] Add an entity list panel (by `entity_id` and tags)
- [ ] Add an entity detail view (transform, renderable type, agent state summary)

### M2 — timeline controls
- [ ] Add play/pause/step controls for timeline playback
- [ ] Add timeline scrubber and current time display

### M2b — scene playback (video-like)
- [ ] Add rewind/fast-forward controls (when enabled by scene policy)
- [ ] Add scene duration display and end-of-scene behavior indicators (loop/stop)

## Introspection
- [x] Add optional Bevy world/entity introspection using `bevy-inspector-egui` behind a feature flag, gated by scene policy
  - [x] Add `bevy-inspector-egui` integration when `features.inspector_egui` is enabled
  - [x] Gate inspector UI on scene policy (scene-level introspection toggle)

### M3 — advanced dev panels
- [ ] Add config/scene reload status panel with last error display
- [ ] Add performance panel (frame time, sim tick time, asset load stats)
