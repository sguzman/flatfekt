#![cfg(feature = "physics_rapier2d")]

use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use flatfekt_assets::resolve::assets_root;
use flatfekt_config::RootConfig;
use flatfekt_runtime::{
  AssetsRootRes,
  ConfigRes,
  EntityMap,
  FlatfektRuntimePlugin,
  ScenePathRes,
  SceneRes
};
use flatfekt_schema::SceneFile;

fn build_rapier_app() -> App {
  let cfg: RootConfig = toml::from_str(
    r#"
[app]
assets_dir = "assets"

[platform]
unix_backend = "wayland"

[render]
backend = "vulkan"

[simulation]
backend = "rapier2d"
enabled = true
deterministic = true
fixed_dt_secs = 0.01
max_catchup_steps = 1
seed = 123
playing = true
"#
  )
  .expect("config parses");
  cfg.validate().expect("config valid");

  let workspace_root = PathBuf::from(
    env!("CARGO_MANIFEST_DIR")
  )
  .join("../..");
  let scene_path = workspace_root
    .join("scenes/physics_test.toml");
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
    plugins
      .disable::<bevy::winit::WinitPlugin>()
      .disable::<bevy::time::TimePlugin>();

  let mut app = App::new();
  app.add_plugins(plugins);
  app.insert_resource(
    Time::<()>::default()
  );
  app.insert_resource(ConfigRes(cfg));
  app.insert_resource(ScenePathRes(
    scene_path
  ));
  app.insert_resource(SceneRes(scene));
  app.insert_resource(AssetsRootRes(
    root
  ));
  app
    .add_plugins(FlatfektRuntimePlugin);

  app.world_mut().run_schedule(Startup);
  app
}

fn ball_pos(app: &App) -> Vec3 {
  let entity_map =
    app.world().resource::<EntityMap>();
  let ball = *entity_map
    .0
    .get("ball")
    .and_then(|v| v.first())
    .expect("ball spawned");
  app
    .world()
    .get::<Transform>(ball)
    .expect("ball has Transform")
    .translation
}

#[test]
fn rapier_repeatable_within_process_same_dt()
 {
  let mut a1 = build_rapier_app();
  let mut a2 = build_rapier_app();

  let dt =
    Duration::from_secs_f32(0.01);

  for _ in 0..50 {
    a1.world_mut()
      .resource_mut::<Time>()
      .advance_by(dt);
    a2.world_mut()
      .resource_mut::<Time>()
      .advance_by(dt);
    a1.world_mut().run_schedule(Update);
    a2.world_mut().run_schedule(Update);
  }

  let p1 = ball_pos(&a1);
  let p2 = ball_pos(&a2);

  let eps = 1e-4;
  assert!(
    (p1.x - p2.x).abs() <= eps
      && (p1.y - p2.y).abs() <= eps
      && (p1.z - p2.z).abs() <= eps,
    "expected repeatable rapier \
     outcome; p1={p1:?} p2={p2:?}"
  );
}
