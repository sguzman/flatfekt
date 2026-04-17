//! Benchmarks for scene load and patch apply performance.
//!
//! Run with: cargo bench -p flatfekt-schema

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use flatfekt_schema::{SceneFile, ScenePatch};

fn repo_root() -> PathBuf {
  let here = Path::new(env!("CARGO_MANIFEST_DIR"));
  here
    .parent()
    .and_then(|p| p.parent())
    .unwrap()
    .to_path_buf()
}

/// Minimal harness — measures wall-clock time for N iterations.
fn bench<F: FnMut()>(
  name: &str,
  iterations: u32,
  mut f: F,
) {
  // Warm-up
  for _ in 0..10 {
    f();
  }

  let start = Instant::now();
  for _ in 0..iterations {
    f();
  }
  let elapsed: Duration = start.elapsed();
  let per_iter = elapsed / iterations;
  println!(
    "bench {name}: {iterations} iters, \
     total={elapsed:?}, per_iter={per_iter:?}"
  );
}

fn main() {
  let demo_path = repo_root().join(
    "tests/fixtures/scenes/demo.toml",
  );
  let shapes_path = repo_root().join(
    "tests/fixtures/scenes/shapes.toml",
  );
  let add_patch_path = repo_root().join(
    "tests/fixtures/patches/add_entity.toml",
  );

  // --- Scene load benchmarks ---
  bench("scene_load_demo", 1_000, || {
    let _ = black_box(
      SceneFile::load_from_path(&demo_path),
    );
  });

  bench("scene_load_shapes", 1_000, || {
    let _ = black_box(
      SceneFile::load_from_path(&shapes_path),
    );
  });

  // --- Patch deserialize benchmark ---
  let patch_toml =
    std::fs::read_to_string(&add_patch_path)
      .expect("add_entity patch fixture");
  bench("patch_deserialize_add", 10_000, || {
    let _ = black_box(
      toml::from_str::<ScenePatch>(&patch_toml),
    );
  });

  // --- Patch apply benchmark ---
  let scene =
    SceneFile::load_from_path(&demo_path)
      .expect("demo scene");
  let patch: ScenePatch =
    toml::from_str(&patch_toml)
      .expect("add patch");

  bench("patch_apply_add", 10_000, || {
    let mut s = black_box(scene.scene.clone());
    if let ScenePatch::Add { entity } = &patch {
      s.entities.push(entity.clone());
    }
    black_box(s);
  });
}
