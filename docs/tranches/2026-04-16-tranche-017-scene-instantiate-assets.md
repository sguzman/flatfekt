# Tranche 017 (2026-04-16) — scene instantiation + asset refs (10 items)

Selected roadmap items (exactly 10):

- [x] Schema (M0): Define color representation (sRGB triples + alpha)
- [x] Schema (M0): Define sprite spec (image ref + size + anchor)
- [x] Schema (M0): Define text spec (string + font ref + size + alignment/anchor)
- [x] Schema (M0): Define camera spec (2D camera params + clear color)
- [x] Assets (M0): Define `AssetRef` type (logical id vs path) and TOML representation
- [x] Assets (M0): Implement asset root directory config (`app.assets_dir`) and path safety rules
- [x] Assets (M0): Implement image and font resolution and load hooks (enough for sprites/text)
- [x] Runtime (M0): Implement instantiation of: camera, sprites, text, basic transforms
- [x] Runtime (M0): Add structured `tracing` spans around config load, scene load, instantiate
- [x] Rendering (M0): Spawn sprites with explicit z-order/layering semantics
