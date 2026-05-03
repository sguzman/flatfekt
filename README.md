# flatfekt

`flatfekt` is a Rust and Bevy-based 2D scene runtime where scenes are declared in TOML and instantiated by an engine layer.

## Intent

Decouple content authoring from engine code so simulations, games, motion-graphics scenes, and reusable content packs can share one scene/runtime model.

## Ambition

The current README, workspace split, scene assets, and roadmap docs all point toward a broad content runtime that can sit somewhere between a game engine, simulation framework, and declarative scene player.

## Current Status

The repo already has apps, crates, scene/assets directories, test coverage, and roadmap structure. It reads like a serious workspace under active product shaping.

## Core Capabilities Or Focus Areas

- Declarative scene/runtime approach backed by Bevy.
- Separate application, CLI, viewer, asset, config, runtime, and schema crates.
- Docs and roadmaps that connect implementation work to intended capabilities.
- Scene and asset directories for content-driven development.
- Workspace-level checks and tooling support.

## Project Layout

- `apps/flatfekt/`: primary application surface for running the flatfekt runtime.
- `apps/flatfekt-cli/`: command-line entrypoints for scripted or headless workflows.
- `apps/flatfekt-viewer/`: viewer-oriented app surface for inspecting scenes and content.
- `crates/flatfekt-assets/`: asset loading and asset-domain support code.
- `crates/flatfekt-config/`: configuration loading and normalization support.
- `crates/flatfekt-runtime/`: core runtime logic for scene execution and simulation.
- `crates/flatfekt-schema/`: schema/model definitions for declarative scenes.
- `crates/flatfekt-workspace-checks/`: workspace-specific validation and consistency checks.
- `apps/`: workspace application entrypoints and user-facing binaries.
- `crates/`: workspace member crates grouped by subsystem.
- `docs/`: project documentation, reference material, and roadmap notes.
- `flatfekt/`: project-specific content or application assets grouped under the product name.
- `scenes/`: scene definitions and content files consumed by the runtime.
- `scripts/`: helper scripts for development, validation, or release workflows.
- `src/`: Rust source for the main crate or application entrypoint.
- `tests/`: automated tests, fixtures, or parity scenarios.
- `Cargo.toml`: crate or workspace manifest and the first place to check for package structure.

## Setup And Requirements

- Rust toolchain.
- Any graphics/audio dependencies required by Bevy on the local platform.
- Project scene and asset files for meaningful runs.

## Build / Run / Test Commands

```bash
cargo build --workspace
cargo test --workspace
cargo run -p flatfekt
```

## Notes, Limitations, Or Known Gaps

- This is a workspace-shaped platform project, so not every crate is equally user-facing.
- The TOML scene model is central to the repo's design identity.

## Next Steps Or Roadmap Hints

- Keep the schema/runtime boundary explicit as more content types are added.
- Use the viewer and CLI apps to prove the runtime is reusable rather than app-specific.
