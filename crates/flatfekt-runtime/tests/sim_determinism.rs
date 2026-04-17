use bevy::prelude::*;
use flatfekt_config::RootConfig;
use flatfekt_runtime::ConfigRes;
use flatfekt_runtime::simulation::{
  SimTick,
  SimulationClock,
  SimulationSeed,
  init_simulation,
  simulation_driver
};

#[test]
fn simulation_steps_are_deterministic_with_fixed_dt()
 {
  let cfg: RootConfig = toml::from_str(
    r#"
      [platform]
      unix_backend = "wayland"

      [render]
      backend = "vulkan"

      [simulation]
      enabled = true
      deterministic = true
      fixed_dt_secs = 0.1
      max_catchup_steps = 4
      seed = 123
      playing = true
    "#
  )
  .expect("config parses");
  cfg.validate().expect("config valid");

  let mut app = App::new();
  app
    .insert_resource(ConfigRes(cfg))
    .add_message::<SimTick>()
    .init_resource::<SimulationClock>()
    .init_resource::<SimulationSeed>()
    .add_systems(
      Startup,
      init_simulation
    )
    .add_systems(
      Update,
      simulation_driver
    );

  app.world_mut().run_schedule(Startup);

  for _ in 0..10 {
    app
      .world_mut()
      .run_schedule(Update);
  }

  let clock = app
    .world()
    .resource::<SimulationClock>(
  );
  assert_eq!(clock.steps_total, 10);
  assert!(
    (clock.t_secs - 1.0).abs() < 1e-6
  );
}
