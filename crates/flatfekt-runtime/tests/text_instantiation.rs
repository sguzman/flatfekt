use std::path::PathBuf;

use bevy::prelude::*;
use flatfekt_assets::resolve::assets_root;
use flatfekt_config::RootConfig;
use flatfekt_runtime::{
  AssetsRootRes,
  ConfigRes,
  FlatfektRuntimePlugin,
  ScenePathRes,
  SceneRes
};
use flatfekt_schema::SceneFile;

#[test]
fn text_entities_instantiate_into_world()
 {
  let workspace_root = PathBuf::from(
    env!("CARGO_MANIFEST_DIR")
  )
  .join("../..");
  let cfg_path = workspace_root.join(
    "tests/fixtures/configs/valid.toml"
  );
  let scene_path = workspace_root.join(
    "tests/fixtures/scenes/demo.toml"
  );

  let cfg = RootConfig::load_from_path(
    cfg_path
  )
  .expect("config loads");
  let scene =
    SceneFile::load_from_path(
      scene_path.clone()
    )
    .expect("scene loads");

  let root = assets_root(&cfg)
    .unwrap_or_else(|_| {
      workspace_root.join("assets")
    });

  let mut plugins =
    DefaultPlugins.build();
  plugins =
    plugins.disable::<bevy::winit::WinitPlugin>();

  let mut app = App::new();
  app
    .add_plugins(plugins)
    .insert_resource(ConfigRes(cfg))
    .insert_resource(ScenePathRes(
      scene_path
    ))
    .insert_resource(SceneRes(scene))
    .insert_resource(AssetsRootRes(
      root
    ))
    .add_plugins(FlatfektRuntimePlugin);

  app.world_mut().run_schedule(Startup);

  let world = app.world_mut();
  let mut q = world.query::<&Text2d>();
  let count = q.iter(world).count();
  assert!(
    count >= 2,
    "expected at least 2 Text2d \
     entities from demo scene, got \
     {count}"
  );
}
