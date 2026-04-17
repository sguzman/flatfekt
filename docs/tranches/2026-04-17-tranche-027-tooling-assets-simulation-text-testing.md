# Tranche 027 (2026-04-17) — tooling CLI + asset reload semantics + simulation scaffold + text/sim determinism tests (10 items)

Selected roadmap items (exactly 10):

- [x] Tooling (M0): Add a CLI subcommand: `validate <scene.toml>` (exit non-zero on errors)
- [x] Tooling (M0): Add a CLI subcommand: `run <scene.toml>` (overrides config scene path)
- [x] Tooling (M0): Add `tracing` output controls via CLI flags (level/filter)
- [x] Assets (M2): Define and implement asset hot reload semantics (with runtime hot reload)
- [x] Assets (M2): Add config knobs for reload debounce and failure policy
- [x] Simulation (M0): Add fixed timestep driver with configurable `dt` and max catch-up steps
- [x] Simulation (M0): Add `tracing` spans around sim tick and system sets
- [x] Simulation (Determinism policy): Add seed routing for stochastic sim systems (no hidden randomness)
- [x] Testing (M2): Add deterministic sim tick tests (state after N ticks equals expected)
- [x] Text (M0): Add tests that load scenes with text and validate component instantiation
