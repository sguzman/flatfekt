use bevy::prelude::*;
use tracing::instrument;

use crate::{
  ConfigRes,
  SceneRes
};

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
  scene: Res<SceneRes>,
  mut clock: ResMut<SimulationClock>,
  mut seed: ResMut<SimulationSeed>,
  mut region: ResMut<SimRegionRes>
) {
  let cfg = &cfg.0;
  tracing::info!(
    sim_cfg = ?cfg.simulation,
    "init_simulation: checking config"
  );
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

  if let Some(sim_spec) =
    &scene.0.scene.simulation
  {
    region.gravity =
      Vec2::from(sim_spec.gravity);
    if let Some(b) = sim_spec.bounds {
      region.bounds = Some(Rect::new(
        b[0], b[1], b[2], b[3]
      ));
    } else {
      region.bounds = None;
    }
  } else {
    region.gravity =
      Vec2::new(0.0, -9.81);
    region.bounds = None;
  }

  tracing::info!(
    ?clock,
    ?region,
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
    commands.write_message(SimTick {
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

#[derive(
  Component, Debug, Clone, Default,
)]
pub struct PhysicsBody {
  pub velocity:    Vec2,
  pub mass:        f32,
  pub restitution: f32,
  pub friction:    f32,
  pub fixed:       bool
}

#[derive(Component, Debug, Clone)]
pub enum Collider {
  Circle { radius: f32 },
  Rect { size: Vec2 }
}

#[derive(
  Resource, Debug, Clone, Default,
)]
pub struct SimRegionRes {
  pub gravity: Vec2,
  pub bounds:  Option<Rect>
}

#[derive(
  Component, Debug, Clone, Default,
)]
pub struct EntityHealth {
  pub current: f32,
  pub max:     f32
}

#[instrument(level = "trace", skip_all)]
pub fn gravity_system(
  mut ticks: MessageReader<SimTick>,
  region: Res<SimRegionRes>,
  mut query: Query<(
    &mut PhysicsBody,
    &mut Transform
  )>
) {
  for tick in ticks.read() {
    let dt = tick.dt_secs;
    let gravity = region.gravity;
    for (mut body, mut tf) in
      query.iter_mut()
    {
      if body.fixed {
        continue;
      }
      body.velocity += gravity * dt;
      tf.translation +=
        body.velocity.extend(0.0) * dt;
    }
  }
}

#[instrument(level = "trace", skip_all)]
pub fn bounds_collision_system(
  mut ticks: MessageReader<SimTick>,
  region: Res<SimRegionRes>,
  mut query: Query<(
    &mut PhysicsBody,
    &mut Transform,
    &Collider
  )>
) {
  let Some(bounds) = region.bounds
  else {
    return;
  };

  for _ in ticks.read() {
    for (mut body, mut tf, collider) in
      query.iter_mut()
    {
      if body.fixed {
        continue;
      }

      match collider {
        | Collider::Circle {
          radius
        } => {
          let pos = tf.translation.xy();
          let mut new_pos = pos;
          let mut hit = false;

          if pos.x - radius
            < bounds.min.x
          {
            new_pos.x =
              bounds.min.x + radius;
            body.velocity.x *=
              -body.restitution;
            hit = true;
          } else if pos.x + radius
            > bounds.max.x
          {
            new_pos.x =
              bounds.max.x - radius;
            body.velocity.x *=
              -body.restitution;
            hit = true;
          }

          if pos.y - radius
            < bounds.min.y
          {
            new_pos.y =
              bounds.min.y + radius;
            body.velocity.y *=
              -body.restitution;
            hit = true;
          } else if pos.y + radius
            > bounds.max.y
          {
            new_pos.y =
              bounds.max.y - radius;
            body.velocity.y *=
              -body.restitution;
            hit = true;
          }

          if hit {
            tf.translation = new_pos
              .extend(tf.translation.z);
          }
        }
        | Collider::Rect {
          size
        } => {
          let pos = tf.translation.xy();
          let half_size = *size * 0.5;
          let mut new_pos = pos;
          let mut hit = false;

          if pos.x - half_size.x
            < bounds.min.x
          {
            new_pos.x = bounds.min.x
              + half_size.x;
            body.velocity.x *=
              -body.restitution;
            hit = true;
          } else if pos.x + half_size.x
            > bounds.max.x
          {
            new_pos.x = bounds.max.x
              - half_size.x;
            body.velocity.x *=
              -body.restitution;
            hit = true;
          }

          if pos.y - half_size.y
            < bounds.min.y
          {
            new_pos.y = bounds.min.y
              + half_size.y;
            body.velocity.y *=
              -body.restitution;
            hit = true;
          } else if pos.y + half_size.y
            > bounds.max.y
          {
            new_pos.y = bounds.max.y
              - half_size.y;
            body.velocity.y *=
              -body.restitution;
            hit = true;
          }

          if hit {
            tf.translation = new_pos
              .extend(tf.translation.z);
          }
        }
      }
    }
  }
}

#[derive(
  bevy::prelude::Event, Clone, Debug,
)]
pub enum SimControl {
  Pause,
  Play,
  Step,
  Reset
}

#[instrument(level = "info", skip_all)]
pub fn sim_control_system(
  control: On<SimControl>,
  mut clock: ResMut<SimulationClock>
) {
  match control.event() {
    | SimControl::Pause => {
      clock.playing = false;
      tracing::info!(
        "simulation paused"
      );
    }
    | SimControl::Play => {
      clock.playing = true;
      tracing::info!(
        "simulation playing"
      );
    }
    | SimControl::Step => {
      clock.accumulator_secs +=
        clock.dt_secs;
      tracing::info!(
        "simulation stepped"
      );
    }
    | SimControl::Reset => {
      clock.t_secs = 0.0;
      clock.steps_total = 0;
      clock.accumulator_secs = 0.0;
      tracing::info!(
        "simulation reset"
      );
    }
  }
}
