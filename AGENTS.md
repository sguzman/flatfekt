# Flatfekt — agent operating rules

This repository is a Rust workspace that builds a TOML-driven 2D Bevy scene runner and simulation environment.

These rules exist to keep the repo organized, buildable, auditable, and roadmap-driven.

## Non-negotiables

- This is a Rust project; prefer stable Rust + stable, well-maintained crates.
- Use extensive structured logging with `tracing` at subsystem boundaries (config load, scene load/validate, asset resolve/load, instantiate, timeline/sim tick, UI boundaries).
- Scenes are controlled via TOML (scene file + tables within it). Project behavior/policy/tunables are controlled via a centralized TOML “control pane”.
- Default Unix graphics environment is **Wayland**.
- Rendering backend is **Vulkan only**: fail fast if Vulkan is not available.
- Never reference the `tmp/` reference projects in docs or code. They are for private developer reference only.

## Roadmaps and process discipline

- Roadmaps live under `docs/roadmaps/`.
- Every actionable roadmap item is a Markdown checkbox (`- [ ]` / `- [x]`).
- Only include work that can be implemented/tested/linted/built/verified by automation in this repo.
  - Do not add manual QA or subjective review tasks as checkbox items.
- Keep roadmap state aligned with the repository:
  - When implementing work, identify the roadmap item(s) first.
  - Check off items only when implemented and verified.
  - If partially done, split into smaller sub-items; do not mark partially complete items as done.
- Avoid inventing unrelated roadmap items. If a new item is needed, add it only as a direct dependency/refinement of an existing requirement.

## Tranches

- For every set of changes, create a tranche file under `docs/tranches/` listing the roadmap items being implemented.
- A tranche must list only real roadmap items (or immediate, explicitly added sub-items).
- Do not “fill a quota” by creating made-up items or post-hoc assigning work to items.
- At tranche end:
  - Update the tranche file checkbox states accurately.
  - Update the corresponding roadmap checkbox states accurately.

## Configuration control pane

Configuration is intentionally centralized and aggressive:

- Project control pane config is located under `.config/flatfekt/` (preferred path: `.config/flatfekt/flatfekt.toml`).
- Cache root is `.cache/flatfekt/`.
  - Logs live under `.cache/flatfekt/logs/`.
  - Per-scene derived artifacts go under `.cache/flatfekt/scene/<scene>/`.

Rules:

- Any policy/parameter/tuning knob/limit/feature-flag/path/threshold that may need adjustment belongs in config.
- Avoid scattering magic numbers in code; surface them in config unless they are strict invariants.
- If something is intentionally hardcoded, document why (API constraint or true invariant).

## Modes and logging

- `app.mode` must support `dev` and `prod`.
- In `dev` mode, the app writes run-scoped timestamped log files under `.cache/flatfekt/logs/` in addition to terminal logs.
- Use `tracing_subscriber` with structured fields and avoid per-frame spam (prefer spans or sampled logs).

## Workspace organization expectations

- Keep Bevy types out of schema/config crates when feasible; schema should remain Bevy-free.
- Maintain a clean dependency DAG: schema/config are foundational; runtime depends on them; apps depend on engine crates.
- Prefer clear module boundaries; keep public API surfaces small and explicit.

## Formatting, linting, and verification

- `rustfmt.toml` is intentionally aggressive; always run formatting after changes.
- After code changes, always verify the project builds.
  - Preferred verification: `cargo fmt` and `cargo test`.
  - If a verification step cannot be run, state that explicitly and why.

## Commit messages

- After each completed implementation pass, provide a plain (non-semver) commit message in the assistant response.
- Do not tag releases or include semver prefixes like `feat:` / `fix:` unless explicitly requested by the repo owner.

## Safety and quality bars

- No placeholder code presented as complete.
- Avoid `unwrap()`/`expect()` in engine paths; use explicit errors with context.
- Prefer explicit types when it improves readability.
- Favor deterministic behavior when a policy exists (ordering, IDs, stable sorting keys).

