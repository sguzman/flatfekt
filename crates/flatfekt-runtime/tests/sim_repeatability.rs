use bevy::prelude::*;
use flatfekt_config::RootConfig;
use flatfekt_runtime::simulation::{
  Collider,
  PhysicsBody,
  SimRegionRes,
  SimTick,
  SimulationClock,
  SimulationSeed,
  bounds_collision_system,
  enforce_sim_determinism_system,
  gravity_system,
  init_simulation,
  simulation_driver
};
use flatfekt_runtime::{
  ConfigRes,
  SceneRes
};
use flatfekt_schema::SceneFile;

fn build_native_sim_app(
  seed: u64
) -> App {
  let cfg: RootConfig = toml::from_str(
    format!(
      r#"
[platform]
unix_backend = "wayland"

[render]
backend = "vulkan"

[simulation]
backend = "native"
enabled = true
deterministic = true
fixed_dt_secs = 0.1
max_catchup_steps = 4
seed = {seed}
playing = true
"#
    )
    .as_str()
  )
  .expect("config parses");
  cfg.validate().expect("config valid");

  let scene_file: SceneFile =
    toml::from_str(
      r#"
[scene]
schema_version = "0.1"
entities = []

[scene.simulation]
gravity = [0.0, -10.0]
bounds = [-100.0, -100.0, 100.0, 100.0]
"#
    )
    .expect("scene parses");

  let mut app = App::new();
  app
    .insert_resource(ConfigRes(cfg))
    .insert_resource(SceneRes(
      scene_file
    ))
    .add_message::<SimTick>()
    .init_resource::<SimulationClock>()
    .init_resource::<SimulationSeed>()
    .init_resource::<SimRegionRes>()
    .init_resource::<flatfekt_runtime::simulation::DeterminismPolicyRes>()
    .add_systems(
      Startup,
      init_simulation
    )
    .add_systems(
      Startup,
      enforce_sim_determinism_system
        .after(init_simulation)
    )
    .add_systems(
      Update,
      simulation_driver
    )
    .add_systems(
      Update,
      (
        gravity_system,
        bounds_collision_system,
      )
        .after(simulation_driver)
    );

  app.world_mut().run_schedule(Startup);

  // Spawn multiple bodies to ensure
  // stable iteration order matters.
  app.world_mut().spawn((
    PhysicsBody {
      velocity:    Vec2::new(1.0, 0.0),
      mass:        1.0,
      restitution: 0.8,
      friction:    0.5,
      fixed:       false
    },
    Collider::Circle {
      radius: 5.0
    },
    Transform::from_xyz(0.0, 80.0, 0.0)
  ));
  app.world_mut().spawn((
    PhysicsBody {
      velocity:    Vec2::new(-1.0, 0.0),
      mass:        2.0,
      restitution: 0.5,
      friction:    0.5,
      fixed:       false
    },
    Collider::Circle {
      radius: 6.0
    },
    Transform::from_xyz(
      10.0, 40.0, 0.0
    )
  ));

  app
}

#[test]
fn native_sim_is_repeatable_given_same_seed_and_dt()
 {
  let mut a1 =
    build_native_sim_app(123);
  let mut a2 =
    build_native_sim_app(123);

  for _ in 0..20 {
    a1.world_mut().run_schedule(Update);
    a2.world_mut().run_schedule(Update);
  }

  let mut p1: Vec<(f32, f32)> =
    Vec::new();
  {
    let world = a1.world_mut();
    let mut q = world
      .query::<(Entity, &Transform)>();
    for (_, tf) in q.iter(world) {
      p1.push((
        tf.translation.x,
        tf.translation.y
      ));
    }
  }
  let mut p2: Vec<(f32, f32)> =
    Vec::new();
  {
    let world = a2.world_mut();
    let mut q = world
      .query::<(Entity, &Transform)>();
    for (_, tf) in q.iter(world) {
      p2.push((
        tf.translation.x,
        tf.translation.y
      ));
    }
  }

  p1.sort_by(|a, b| {
    a.0.partial_cmp(&b.0).unwrap()
  });
  p2.sort_by(|a, b| {
    a.0.partial_cmp(&b.0).unwrap()
  });

  assert_eq!(p1, p2);
}
