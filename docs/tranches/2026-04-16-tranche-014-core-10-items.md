# Tranche 014 (2026-04-16) — core roadmap (10 items)

Selected roadmap items (exactly 10):

- [x] Define which crates are allowed to touch Bevy types directly (prefer keeping “schema” crates Bevy-free)
- [x] Define a single config entrypoint (`flatfekt.toml`) and lookup rules (cwd, env override)
- [x] Support default config directory `.config/flatfekt/` (prefer `.config/flatfekt/flatfekt.toml`)
- [x] Add `deny`/`warn` lints in `Cargo.toml` or `.cargo/config.toml` (minimal, practical)
- [x] Add `cargo fmt` + `cargo clippy` + `cargo test` command set in root `README.md`
- [x] Add a small “engine bootstrap” app demonstrating config load + tracing init + scene load
- [x] Add `app.mode` config (`dev`|`prod`) and wire it to operational behavior
- [x] In `dev` mode, write run-scoped timestamped log files under `.cache/flatfekt/logs/` in addition to terminal output
- [x] Standardize cache directory layout under `.cache/flatfekt/` (central cache root)
- [x] Create per-scene cache directories under `.cache/flatfekt/scene/<scene>/` for derived artifacts
