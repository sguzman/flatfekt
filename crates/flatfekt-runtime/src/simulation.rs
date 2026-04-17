use bevy::prelude::*;
use tracing::instrument;

use crate::ConfigRes;

#[derive(Resource, Debug, Clone)]
pub struct SimulationSeed(pub u64);

impl Default for SimulationSeed {
  fn default() -> Self {
    Self(0)
  }
}

impl SimulationSeed {
  pub fn next_u64(&mut self) -> u64 {
    // splitmix64: fast, deterministic,
    // and dependency-free
    let mut z = self
      .0
      .wrapping_add(0x9e3779b97f4a7c15);
    self.0 = z;
    z = (z ^ (z >> 30))
      .wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27))
      .wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
  }
}

#[derive(Resource, Debug, Clone)]
pub struct SimulationClock {
  pub enabled:           bool,
  pub playing:           bool,
  pub t_secs:            f32,
  pub dt_secs:           f32,
  pub max_catchup_steps: u32,
  pub accumulator_secs:  f32,
  pub steps_total:       u64
}

impl Default for SimulationClock {
  fn default() -> Self {
    Self {
      enabled:           false,
      playing:           true,
      t_secs:            0.0,
      dt_secs:           1.0 / 60.0,
      max_catchup_steps: 4,
      accumulator_secs:  0.0,
      steps_total:       0
    }
  }
}

#[derive(
  Message,
  bevy::prelude::Event,
  Clone,
  Debug,
)]
pub struct SimTick {
  pub dt_secs: f32
}

#[instrument(level = "info", skip_all)]
pub fn init_simulation(
  cfg: Res<ConfigRes>,
  mut clock: ResMut<SimulationClock>,
  mut seed: ResMut<SimulationSeed>
) {
  let cfg = &cfg.0;
  clock.enabled =
    cfg.simulation_enabled();
  clock.playing =
    cfg.simulation_playing();
  clock.dt_secs =
    cfg.simulation_fixed_dt_secs();
  clock.max_catchup_steps =
    cfg.simulation_max_catchup_steps();
  clock.t_secs = 0.0;
  clock.accumulator_secs = 0.0;
  clock.steps_total = 0;
  seed.0 = cfg.simulation_seed();
  tracing::info!(
    ?clock,
    seed = seed.0,
    "initialized simulation"
  );
}

#[instrument(level = "trace", skip_all)]
pub fn simulation_driver(
  time: Option<Res<Time>>,
  cfg: Res<ConfigRes>,
  mut clock: ResMut<SimulationClock>,
  mut commands: Commands
) {
  if !clock.enabled {
    return;
  }
  if !clock.playing {
    return;
  }

  let dt =
    if cfg.0.simulation_deterministic()
    {
      clock.dt_secs
    } else {
      time
        .map(|t| t.delta_secs())
        .unwrap_or(clock.dt_secs)
    };

  if dt.is_finite() && dt > 0.0 {
    clock.accumulator_secs += dt;
  }

  let mut steps: u32 = 0;
  while clock.accumulator_secs
    >= clock.dt_secs
    && steps < clock.max_catchup_steps
  {
    clock.t_secs += clock.dt_secs;
    clock.accumulator_secs -=
      clock.dt_secs;
    clock.steps_total += 1;
    steps += 1;
    commands.trigger(SimTick {
      dt_secs: clock.dt_secs
    });
  }

  if steps == clock.max_catchup_steps
    && clock.accumulator_secs
      >= clock.dt_secs
  {
    tracing::warn!(
      accumulator_secs =
        clock.accumulator_secs,
      "simulation catch-up cap hit; \
       dropping accumulated time"
    );
    clock.accumulator_secs = 0.0;
  }
}

#[derive(Component, Debug, Clone, Default)]
pub struct PhysicsBody {
  pub velocity: Vec2,
  pub mass:     f32
}

#[derive(Component, Debug, Clone, Default)]
pub struct EntityHealth {
  pub current: f32,
  pub max:     f32
}

#[instrument(level = "trace", skip_all)]
pub fn gravity_system(
  tick: Trigger<SimTick>,
  mut query: Query<(&mut PhysicsBody, &mut Transform)>
) {
  let dt = tick.event().dt_secs;
  // Stub gravity vector
  let gravity = Vec2::new(0.0, -9.81);
  for (mut body, mut tf) in query.iter_mut() {
    body.velocity += gravity * dt;
    tf.translation += body.velocity.extend(0.0) * dt;
  }
}
