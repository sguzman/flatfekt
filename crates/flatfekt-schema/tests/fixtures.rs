use std::path::{
  Path,
  PathBuf
};

use flatfekt_schema::SceneFile;

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
