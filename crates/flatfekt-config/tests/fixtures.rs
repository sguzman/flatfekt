use std::path::{
  Path,
  PathBuf
};

use flatfekt_config::RootConfig;

fn repo_root() -> PathBuf {
  let here = Path::new(env!(
    "CARGO_MANIFEST_DIR"
  ));
  here
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .to_path_buf()
}

#[test]
fn valid_config_loads() {
  let path = repo_root().join(
    "tests/fixtures/configs/valid.toml"
  );
  let cfg =
    RootConfig::load_from_path(path);
  assert!(cfg.is_ok(), "{cfg:?}");
}

#[test]
fn invalid_asset_map_path_rejected() {
  let path = repo_root().join(
    "tests/fixtures/configs/\
     invalid_asset_path.toml"
  );
  let cfg =
    RootConfig::load_from_path(path);
  assert!(cfg.is_err());
}
