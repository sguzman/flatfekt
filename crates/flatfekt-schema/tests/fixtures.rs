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
