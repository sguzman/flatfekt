# Tranche 026 — Testing + Tooling

## Roadmap Items

### Testing
- [x] Add golden patch fixtures (`tests/fixtures/patches/*.toml`)
- [x] Add `patches` fixture test to flatfekt-schema
- [x] Add benchmarks for scene load/instantiate time
- [x] Add benchmarks for hot reload apply time and patch apply time

### Tooling CLI
- [x] Create `apps/flatfekt-cli`
- [x] Implement `validate` subcommand
- [x] Implement `run` subcommand
- [x] Implement `fmt` subcommand
- [x] Implement `resolve` subcommand
- [x] Implement `migrate` subcommand (stub)
- [x] Implement `diff` subcommand (basic)
- [x] Implement `new` subcommand
- [x] Implement `demo` subcommand
- [x] Add tracing controls via CLI flags

## Verification Results

- `cargo test -p flatfekt-schema` passes with 10 tests.
- `cargo build -p flatfekt-cli` compiles successfully.
- `flatfekt validate tests/fixtures/scenes/demo.toml` returns OK.
- `flatfekt new temp_scene.toml` creates a valid template.
- `flatfekt resolve tests/fixtures/scenes/demo.toml` prints pretty TOML.
