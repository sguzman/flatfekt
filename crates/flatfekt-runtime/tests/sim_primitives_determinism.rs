use std::time::Duration;

use bevy::prelude::*;
use flatfekt_config::RootConfig;
use flatfekt_runtime::simulation::{
  Grid,
  GridRule,
  Particle,
  ParticleSystem,
  SimRegionRes,
  SimTick,
  SimulationClock,
  SimulationSeed,
  grid_tick,
  init_simulation,
  particle_system_tick,
  simulation_driver
};
use flatfekt_runtime::{
  ConfigRes,
  SceneRes
};
use flatfekt_schema::SceneFile;

#[test]
fn particle_system_determinism() {
  let cfg: RootConfig = toml::from_str(
    r#"
      [platform]
      unix_backend = "wayland"
      [render]
      backend = "vulkan"
      [simulation]
      enabled = true
      playing = true
      deterministic = true
      fixed_dt_secs = 0.1
      seed = 42
    "#
  )
  .unwrap();

  let mut app = App::new();
  let mut plugins =
    MinimalPlugins.build();
  plugins = plugins
    .disable::<bevy::time::TimePlugin>(
  );
  app.add_plugins(plugins);
  app.insert_resource(
    Time::<()>::default()
  );
  // TransformPlugin is needed for
  // GlobalTransform
  app.add_plugins(TransformPlugin);

  app
    .insert_resource(ConfigRes(cfg))
    .insert_resource(SceneRes(
      toml::from_str::<SceneFile>(
        r#"
[scene]
schema_version = "0.1"
entities = []
"#
      )
      .expect("scene parses")
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
      Update,
      simulation_driver
    )
    .add_systems(
      Update,
      (particle_system_tick)
        .after(simulation_driver)
    );

  app.world_mut().run_schedule(Startup);

  // Spawn a particle system
  app.world_mut().spawn((
    ParticleSystem {
      emission_rate: 100.0, // High rate to ensure we spawn something in 0.5s
      lifetime: 1.0,
      velocity_min: Vec2::new(-1.0, -1.0),
      velocity_max: Vec2::new(1.0, 1.0),
      max_particles: 1000,
      accumulator: 0.0,
    },
    Transform::default(),
    GlobalTransform::default(),
  ));

  // Run for some steps
  for _ in 0..5 {
    app
      .world_mut()
      .resource_mut::<Time>()
      .advance_by(
        Duration::from_secs_f32(0.1)
      );
    app.update();
  }

  // Count particles
  {
    let mut query = app
      .world_mut()
      .query::<&Particle>(
    );
    let count1 =
      query.iter(app.world()).count();
    assert!(
      count1 > 0,
      "Should have spawned particles"
    );

    // Run another 5 steps
    for _ in 0..5 {
      app
        .world_mut()
        .resource_mut::<Time>()
        .advance_by(
          Duration::from_secs_f32(0.1)
        );
      app.update();
    }
    let count2 =
      query.iter(app.world()).count();
    assert!(
      count2 > count1,
      "Should have spawned more \
       particles"
    );
  }
}

#[test]
fn grid_ca_determinism() {
  let cfg: RootConfig = toml::from_str(
    r#"
      [platform]
      unix_backend = "wayland"
      [render]
      backend = "vulkan"
      [simulation]
      enabled = true
      playing = true
      deterministic = true
      fixed_dt_secs = 1.0
      seed = 123
    "#
  )
  .unwrap();

  let mut app = App::new();
  let mut plugins =
    MinimalPlugins.build();
  plugins = plugins
    .disable::<bevy::time::TimePlugin>(
  );
  app.add_plugins(plugins);
  app.insert_resource(
    Time::<()>::default()
  );
  app
    .insert_resource(ConfigRes(cfg))
    .insert_resource(SceneRes(
      toml::from_str::<SceneFile>(
        r#"
[scene]
schema_version = "0.1"
entities = []
"#
      )
      .expect("scene parses")
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
      Update,
      simulation_driver
    )
    .add_systems(
      Update,
      (grid_tick)
        .after(simulation_driver)
    );

  app.world_mut().run_schedule(Startup);

  // Spawn a grid with a blinker pattern
  // (3 cells in a row)
  let mut cells = vec![0u8; 16];
  cells[5] = 1;
  cells[6] = 1;
  cells[7] = 1;

  app.world_mut().spawn(Grid {
    width:      4,
    height:     4,
    cell_size:  1.0,
    cells:      cells.clone(),
    next_cells: vec![0u8; 16],
    rule:       GridRule::Conway
  });

  // Tick 1
  app
    .world_mut()
    .resource_mut::<Time>()
    .advance_by(
      Duration::from_secs_f32(1.0)
    );
  app.update();

  {
    let mut query =
      app.world_mut().query::<&Grid>();
    let grid = query
      .iter(app.world())
      .next()
      .expect(
        "Should have exactly one grid"
      );

    // Blinker should rotate
    assert_eq!(grid.cells[2], 1);
    assert_eq!(grid.cells[6], 1);
    assert_eq!(grid.cells[10], 1);
    assert_eq!(grid.cells[5], 0);
    assert_eq!(grid.cells[7], 0);
  }

  // Tick 2
  app
    .world_mut()
    .resource_mut::<Time>()
    .advance_by(
      Duration::from_secs_f32(1.0)
    );
  app.update();

  {
    let mut query =
      app.world_mut().query::<&Grid>();
    let grid = query
      .iter(app.world())
      .next()
      .expect(
        "Should have exactly one grid"
      );

    // Should be back to horizontal
    assert_eq!(grid.cells[5], 1);
    assert_eq!(grid.cells[6], 1);
    assert_eq!(grid.cells[7], 1);
    assert_eq!(grid.cells[2], 0);
    assert_eq!(grid.cells[10], 0);
  }
}
