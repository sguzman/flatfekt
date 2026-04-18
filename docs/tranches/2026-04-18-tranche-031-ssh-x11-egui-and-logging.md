# Tranche 031 (2026-04-18) — SSH X11 forwarding stability: egui scheduling + disable Bevy LogPlugin (2 items)

Selected roadmap items (exactly 2):

- [x] UI (Egui): Ensure the egui control panel runs between `BeginPass` and `EndPass` (avoid “No fonts available…” panic)
- [x] Core (Logging): Avoid double-initializing the global logger (disable Bevy `LogPlugin` when using `tracing_subscriber`)
