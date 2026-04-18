use std::path::{
  Path,
  PathBuf
};

use flatfekt_config::{
  AppConfig,
  RootConfig
};
use flatfekt_runtime::bake::{
  BAKE_JSON_FILE,
  BAKE_SCENE_PLAYBACK_TOML,
  BakeRequest,
  BakedSimulation,
  bake_output_dir_for_scene,
  bake_scene_to_dir
};
use flatfekt_schema::SceneFile;

#[test]
fn bake_dir_builder_matches_spec() {
  let output_root =
    Path::new(".cache/flatfekt/scene");
  let scene_path = Path::new(
    "scenes/physics_test.toml"
  );
  let dir = bake_output_dir_for_scene(
    output_root,
    scene_path,
    "deadbeef",
    123,
    999
  );
  let s = dir.to_string_lossy();
  assert!(
    s.contains(
      ".cache/flatfekt/scene/\
       physics_test/bakes/deadbeef/\
       run-123-999"
    ),
    "unexpected dir: {s}"
  );
}

#[test]
fn headless_bake_writes_json_and_moves_ball()
-> anyhow::Result<()> {
  let output_root =
    tempfile::tempdir()?;

  let scene_path = PathBuf::from(env!(
    "CARGO_MANIFEST_DIR"
  ))
  .join(
    "../../scenes/physics_test.toml"
  );
  let scene_bytes =
    std::fs::read(&scene_path)?;
  let scene_file =
    SceneFile::load_from_path(
      &scene_path
    )?;

  let cfg = RootConfig::default();
  let out = bake_scene_to_dir(
    cfg,
    scene_path,
    scene_bytes,
    scene_file,
    BakeRequest {
      output_root:   output_root
        .path()
        .to_path_buf(),
      fps:           30.0,
      duration_secs: 0.2,
      copy_assets:   true
    }
  )?;

  assert!(out.bake_json_path.exists());
  assert!(
    out.scene_playback_path.exists()
  );
  assert!(
    out
      .dir
      .join(BAKE_JSON_FILE)
      .exists()
  );
  assert!(
    out
      .dir
      .join(BAKE_SCENE_PLAYBACK_TOML)
      .exists()
  );

  let json = std::fs::read_to_string(
    &out.bake_json_path
  )?;
  let baked: BakedSimulation =
    serde_json::from_str(&json)?;

  let ball = baked
    .entities
    .get("ball")
    .ok_or_else(|| {
      anyhow::anyhow!(
        "missing baked entity 'ball'"
      )
    })?;
  assert!(
    ball.keyframes.len() > 1,
    "expected >1 keyframe, got {}",
    ball.keyframes.len()
  );
  let y0 = ball
    .keyframes
    .first()
    .unwrap()
    .transform
    .y;
  let y1 = ball
    .keyframes
    .last()
    .unwrap()
    .transform
    .y;
  assert!(
    y1 < y0,
    "expected ball to fall: y0={y0} \
     y1={y1}"
  );

  Ok(())
}

#[test]
fn bake_packages_assets_and_rewrites_scene_paths()
-> anyhow::Result<()> {
  let root = tempfile::tempdir()?;
  let src_assets =
    root.path().join("src_assets");
  let scene_dir =
    root.path().join("scenes");
  std::fs::create_dir_all(&src_assets)?;
  std::fs::create_dir_all(&scene_dir)?;

  let img_rel =
    PathBuf::from("images/test.png");
  let img_abs =
    src_assets.join(&img_rel);
  std::fs::create_dir_all(
    img_abs.parent().unwrap()
  )?;
  std::fs::write(
    &img_abs,
    b"not-a-real-png"
  )?;

  let scene_path =
    scene_dir.join("one.toml");
  std::fs::write(
    &scene_path,
    r#"
[scene]
schema_version = "0.1"

[[scene.entities]]
id = "sprite"
transform = { x = 0.0, y = 0.0 }
sprite = { image = { path = "images/test.png" }, width = 10.0, height = 10.0 }
"#
  )?;
  let scene_bytes =
    std::fs::read(&scene_path)?;
  let scene_file =
    SceneFile::load_from_path(
      &scene_path
    )?;

  let mut cfg = RootConfig::default();
  cfg.app = Some(AppConfig {
    assets_dir: Some(
      src_assets.clone()
    ),
    ..Default::default()
  });

  let out = bake_scene_to_dir(
    cfg,
    scene_path.clone(),
    scene_bytes,
    scene_file,
    BakeRequest {
      output_root:   root
        .path()
        .join("out"),
      fps:           10.0,
      duration_secs: 0.1,
      copy_assets:   true
    }
  )?;

  let packaged = out
    .dir
    .join("assets")
    .join(&img_rel);
  assert!(
    packaged.exists(),
    "expected packaged asset at {}",
    packaged.display()
  );

  let playback_scene =
    SceneFile::load_from_path(
      &out.scene_playback_path
    )?;
  let sprite = playback_scene
    .scene
    .entities
    .iter()
    .find(|e| e.id == "sprite")
    .and_then(|e| e.sprite.as_ref())
    .ok_or_else(|| {
      anyhow::anyhow!(
        "missing sprite entity"
      )
    })?;
  let path = match &sprite.image {
    | flatfekt_schema::AssetRef::Path {
      path
    } => path.clone(),
    | other => {
      anyhow::bail!(
        "expected AssetRef::Path, got {other:?}"
      )
    }
  };
  assert_eq!(
    path,
    PathBuf::from("assets")
      .join(img_rel)
  );

  Ok(())
}
