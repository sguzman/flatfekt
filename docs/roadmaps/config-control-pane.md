# flatfekt — TOML control-pane config roadmap

This roadmap tracks the project’s centralized configuration surface. Each checkbox corresponds to a concrete, implemented config field with wiring and validation (not just a documented idea).

Target file: `flatfekt.toml` (exact path to be finalized in tranche 1).

## Core

- [ ] `app.name` (string)
- [ ] `app.scene_path` (path; entrypoint scene TOML)
- [ ] `app.assets_dir` (path)
- [ ] `logging.level` (enum/string)
- [ ] `logging.filter` (string; optional override)

## Window / Render

- [ ] `window.title` (string)
- [ ] `window.vsync` (bool or enum mapping to Bevy `PresentMode`)
- [ ] `window.width` / `window.height` (u32; optional)
- [ ] `window.clear_color` (RGB or SRGB triple)

## World / UI

- [ ] `world.background_size` (Vec2)
- [ ] `ui.title_text` (string)
- [ ] `ui.help_text` (string)
- [ ] `ui.title_font_size` / `ui.help_font_size` (f32)
- [ ] `ui.margins_px` (u32)
- [ ] `features.ui_overlay` (bool)

## Scene

- [ ] `scene.entities` (array; typed entity specs)
- [ ] `scene.defaults` (table; common defaults like fonts/colors)

## Citizens

- [ ] `citizens.count` (usize)
- [ ] `citizens.seed` (u64)
- [ ] `citizens.min_sides` / `citizens.max_sides` (u32)
- [ ] `citizens.base_radius` / `citizens.radius_step` (f32)
- [ ] `citizens.spawn_ring_radius` (f32)
- [ ] `citizens.world_radius` (f32)
- [ ] `citizens.wander_params` (structured; wobble/drift tuning)
- [ ] `citizens.tint_params` (structured; saturation/lightness tuning)

## Player

- [ ] `player.size` (f32 or Vec2)
- [ ] `player.max_speed` (f32)
- [ ] `player.accel` / `player.decel` (f32)
- [ ] `player.deadzone` (f32)
- [ ] `player.reset_button` (enum; gamepad mapping)
- [ ] `player.world_radius` (f32)

## Hot Reload

- [ ] `features.hot_reload` (bool)
- [ ] `hot_reload.debounce_ms` (u64)

## Timeline / Transitions

- [ ] `timeline.enabled` (bool)
- [ ] `timeline.fixed_dt_secs` (f32; optional)
- [ ] `timeline.events_path` (path; optional directory of patches)
