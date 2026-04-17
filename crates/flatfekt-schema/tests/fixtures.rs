use std::path::{
  Path,
  PathBuf
};

use flatfekt_schema::{SceneFile, ScenePatch};

fn repo_root() -> PathBuf {
  let here = Path::new(env!(
    "CARGO_MANIFEST_DIR"
  ));
  here
    .parent()
    .and_then(|p| p.parent())
    .unwrap()
    .to_path_buf()
}

#[test]
fn fixture_demo_loads() {
  let path = repo_root().join(
    "tests/fixtures/scenes/demo.toml"
  );
  let file =
    SceneFile::load_from_path(&path);
  assert!(file.is_ok(), "{file:?}");
}

#[test]
fn fixture_shapes_loads() {
  let path = repo_root().join(
    "tests/fixtures/scenes/shapes.toml"
  );
  let file =
    SceneFile::load_from_path(&path);
  assert!(file.is_ok(), "{file:?}");
}

#[test]
fn fixture_unknown_fields_rejected() {
  let path = repo_root().join(
    "tests/fixtures/scenes/\
     unknown_field.toml"
  );
  let file =
    SceneFile::load_from_path(&path);
  assert!(file.is_err());
}

#[test]
fn all_scene_fixtures_load_or_fail_as_expected()
 {
  let dir = repo_root()
    .join("tests/fixtures/scenes");
  for entry in
    std::fs::read_dir(dir).unwrap()
  {
    let entry = entry.unwrap();
    let path = entry.path();
    if path
      .extension()
      .and_then(|s| s.to_str())
      != Some("toml")
    {
      continue;
    }
    let name = path
      .file_name()
      .unwrap()
      .to_string_lossy();
    let res =
      SceneFile::load_from_path(&path);
    let should_fail = name
      .starts_with("invalid_")
      || name.contains("unknown_field");
    if should_fail {
      assert!(
        res.is_err(),
        "{name} should fail but loaded"
      );
    } else {
      assert!(
        res.is_ok(),
        "{name} should load: {res:?}"
      );
    }
  }
}

// ── Patch fixture tests ──────────────────

fn load_patch(name: &str) -> ScenePatch {
  let path = repo_root().join(format!(
    "tests/fixtures/patches/{name}"
  ));
  let raw =
    std::fs::read_to_string(&path)
      .unwrap_or_else(|e| {
        panic!(
          "read patch {name}: {e}"
        )
      });
  toml::from_str::<ScenePatch>(&raw)
    .unwrap_or_else(|e| {
      panic!(
        "parse patch {name}: {e}"
      )
    })
}

#[test]
fn patch_add_entity_deserializes() {
  let p = load_patch("add_entity.toml");
  assert!(
    matches!(p, ScenePatch::Add { .. }),
    "expected Add, got {p:?}"
  );
}

#[test]
fn patch_remove_entity_deserializes() {
  let p =
    load_patch("remove_entity.toml");
  assert!(
    matches!(p, ScenePatch::Remove { .. }),
    "expected Remove, got {p:?}"
  );
}

#[test]
fn patch_update_transform_deserializes() {
  let p =
    load_patch("update_transform.toml");
  assert!(
    matches!(p, ScenePatch::Update { .. }),
    "expected Update, got {p:?}"
  );
}

#[test]
fn patch_invalid_op_rejected() {
  let path = repo_root().join(
    "tests/fixtures/patches/\
     invalid_op.toml"
  );
  let raw =
    std::fs::read_to_string(path).unwrap();
  let res =
    toml::from_str::<ScenePatch>(&raw);
  assert!(
    res.is_err(),
    "invalid op should be rejected"
  );
}

#[test]
fn patch_add_applies_to_scene() {
  let scene_path = repo_root().join(
    "tests/fixtures/scenes/demo.toml"
  );
  let mut scene =
    SceneFile::load_from_path(&scene_path)
      .expect("demo scene")
      .scene;
  let before = scene.entities.len();
  let patch = load_patch("add_entity.toml");
  if let ScenePatch::Add { entity } = patch {
    scene.entities.push(entity);
  }
  assert_eq!(
    scene.entities.len(),
    before + 1,
    "entity count should increase by 1"
  );
}

#[test]
fn patch_remove_applies_to_scene() {
  let scene_path = repo_root().join(
    "tests/fixtures/scenes/demo.toml"
  );
  let mut scene =
    SceneFile::load_from_path(&scene_path)
      .expect("demo scene")
      .scene;
  let before = scene.entities.len();
  let patch =
    load_patch("remove_entity.toml");
  if let ScenePatch::Remove { entity_id } =
    patch
  {
    scene.entities.retain(|e| {
      e.id != entity_id
    });
  }
  assert!(
    scene.entities.len() < before,
    "entity count should decrease"
  );
}
