# Tranche 026 (2026-04-17) — rendering effects + WGSL + render-to-texture (10 items)

Selected roadmap items (exactly 10):

- [x] Rendering (M2): Add simple shader/material hooks (optional; feature-gated)
- [x] Rendering (M2): Add layered post-processing pipeline hooks (optional; future-ready)
- [x] Rendering (Effect integration): Define TOML-facing effect binding model (per-entity and/or global passes)
- [x] Rendering (Effect integration): Add WGSL effect material example (minimal) and ensure it loads from TOML refs
- [x] Rendering (M3): Add render-to-texture support for compositing
- [x] Assets (Shaders/WGSL): Support WGSL shader assets (materials/effects) referenced from TOML
- [x] Assets (Shaders/WGSL): Add shader compilation/validation error reporting with actionable paths
- [x] Assets (Shaders/WGSL): Add shader hot-reload integration (ties into runtime hot reload)
- [x] Testing (M0): Add smoke test that instantiates a minimal scene into a Bevy `App` (headless if possible)
- [x] Testing (M2): Add deterministic timeline tests (values at timestamps equal expected)
