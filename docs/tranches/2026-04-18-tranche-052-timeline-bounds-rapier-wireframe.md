# Tranche 052 — Timeline scrub behavior + bounds + Rapier wireframe

## Roadmap Items
### `ui-roadmap.md`
- [x] Ensure debug toggles render visuals (wireframe/bounds)
- [x] Update scrubber to apply frames while paused (baked playback seeks must redraw)
- [x] Add seek-to-start / seek-to-end buttons
- [x] Add mousewheel seek on scrubber hover (Ctrl+wheel adjusts seek step)

### `simulation-roadmap.md`
- [x] Integrate Rapier2D backend behind `physics_rapier2d` (runtime plugin + fixed timestep wiring)
- [x] Map schema rigid bodies/colliders to Rapier components for `scenes/physics_test.toml`
- [x] Wire `DebugSettings.wireframe` to Rapier debug renderer (collider outlines + axes)
