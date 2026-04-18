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
  pub steps_total:       u64,
  pub time_scale:        f32
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
      steps_total:       0,
      time_scale:        1.0
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
  clock.time_scale =
    cfg.simulation_time_scale();
  seed.0 = cfg.simulation_seed();

  if scene.0.scene.baked.is_some()
    && cfg
      .runtime_playback_prefer_baked_over_simulation()
  {
    clock.enabled = false;
    clock.playing = false;
    tracing::info!(
      "simulation disabled (baked playback mode)"
    );
  }

  // 1. Base values from global config
  region.gravity =
    Vec2::new(0.0, -9.81);
  region.bounds = None;
  region.time_scale =
    cfg.simulation_time_scale();

  // 2. Overrides from scene spec
  if let Some(sim_spec) =
    &scene.0.scene.simulation
  {
    if let Some(g) = sim_spec.gravity {
      region.gravity = Vec2::from(g);
    }
    if let Some(b) = sim_spec.bounds {
      region.bounds = Some(Rect::new(
        b[0], b[1], b[2], b[3]
      ));
    }
    if let Some(ts) =
      sim_spec.time_scale
    {
      region.time_scale = ts;
    }
  }

  // Sync clock time_scale with region
  // (per-scene override wins)
  clock.time_scale = region.time_scale;

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

  let delta = time
    .map(|t| t.delta_secs())
    .unwrap_or(clock.dt_secs)
    * clock.time_scale;

  if !cfg.0.simulation_deterministic() {
    // Non-deterministic: one step with
    // scaled delta
    clock.t_secs += delta;
    clock.steps_total += 1;
    commands.write_message(SimTick {
      dt_secs: delta
    });
    return;
  }

  // Deterministic: accumulate scaled
  // delta and run fixed steps
  if delta.is_finite() && delta > 0.0 {
    clock.accumulator_secs += delta;
  }

  let mut steps: u32 = 0;
  while clock.accumulator_secs
    >= clock.dt_secs
    && steps < clock.max_catchup_steps
  {
    clock.accumulator_secs -=
      clock.dt_secs;
    clock.t_secs += clock.dt_secs;
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
  pub gravity:    Vec2,
  pub bounds:     Option<Rect>,
  pub time_scale: f32
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

pub fn draw_physics_debug_system(
  scene: Res<crate::SceneRes>,
  region: Res<SimRegionRes>,
  settings: Res<crate::DebugSettings>,
  query: Query<(&Transform, &Collider)>,
  mut gizmos: Gizmos
) {
  let introspection = scene
    .0
    .scene
    .playback
    .as_ref()
    .and_then(|p| {
      p.enable_introspection
    })
    .unwrap_or(false);

  if !introspection {
    return;
  }

  // Draw simulation bounds
  if settings.draw_bounds {
    if let Some(bounds) = region.bounds
    {
      gizmos.rect_2d(
        bounds.center(),
        bounds.size(),
        Color::srgba(
          0.0, 1.0, 0.0, 0.3
        )
      );
    }
  }

  // Draw entity colliders
  for (tf, collider) in query.iter() {
    match collider {
      | Collider::Circle {
        radius
      } => {
        gizmos.circle_2d(
          tf.translation.xy(),
          *radius,
          Color::WHITE
        );
      }
      | Collider::Rect {
        size
      } => {
        gizmos.rect_2d(
          tf.translation.xy(),
          *size,
          Color::WHITE
        );
      }
    }
  }
}

pub fn draw_wireframe_system(
  settings: Res<crate::DebugSettings>,
  query: Query<(&Mesh2d, &Transform)>,
  meshes: Res<Assets<Mesh>>,
  mut gizmos: Gizmos
) {
  if !settings.wireframe {
    return;
  }

  for (mesh2d, tf) in query.iter() {
    if let Some(mesh) =
      meshes.get(&mesh2d.0)
    {
      let (indices, positions) = match (
        mesh.indices(),
        mesh.attribute(
          Mesh::ATTRIBUTE_POSITION
        ),
      ) {
        | (
          Some(indices),
          Some(
            bevy_mesh::VertexAttributeValues::Float32x3(
              pos,
            ),
          ),
        ) => (indices, pos),
        | _ => continue,
      };

      let matrix = tf.to_matrix();
      let mut draw_edge =
        |i1: u32, i2: u32| {
          let p1 =
            positions[i1 as usize];
          let p2 =
            positions[i2 as usize];
          let v1 = matrix
            .transform_point3(
              Vec3::from(p1)
            )
            .xy();
          let v2 = matrix
            .transform_point3(
              Vec3::from(p2)
            )
            .xy();
          gizmos.line_2d(
            v1,
            v2,
            Color::srgba(
              1.0, 1.0, 1.0, 0.5
            )
          );
        };

      match indices {
        | bevy_mesh::Indices::U16(
          idx
        ) => {
          for chunk in idx.chunks(3) {
            if chunk.len() == 3 {
              draw_edge(
                chunk[0] as u32,
                chunk[1] as u32
              );
              draw_edge(
                chunk[1] as u32,
                chunk[2] as u32
              );
              draw_edge(
                chunk[2] as u32,
                chunk[0] as u32
              );
            }
          }
        }
        | bevy_mesh::Indices::U32(
          idx
        ) => {
          for chunk in idx.chunks(3) {
            if chunk.len() == 3 {
              draw_edge(
                chunk[0], chunk[1]
              );
              draw_edge(
                chunk[1], chunk[2]
              );
              draw_edge(
                chunk[2], chunk[0]
              );
            }
          }
        }
      }
    }
  }
}

pub fn entity_collision_system(
  mut ticks: MessageReader<SimTick>,
  mut set: ParamSet<(
    Query<(
      Entity,
      &mut PhysicsBody,
      &mut Transform,
      &Collider
    )>,
    Query<(
      Entity,
      &Transform,
      &Collider
    )>
  )>
) {
  for _tick in ticks.read() {
    let mut collisions = Vec::new();

    // 1. Collect all potential
    //    colliders
    let static_colliders: Vec<(
      Entity,
      Vec2,
      Collider
    )> = set
      .p1()
      .iter()
      .map(|(e, tf, col)| {
        (
          e,
          tf.translation.xy(),
          col.clone()
        )
      })
      .collect();

    // 2. Detect collisions
    {
      let mut query = set.p0();
      for (
        entity_a,
        body_a,
        tf_a,
        col_a
      ) in query.iter_mut()
      {
        if body_a.fixed {
          continue;
        }

        let pos_a =
          tf_a.translation.xy();

        for (entity_b, pos_b, col_b) in
          &static_colliders
        {
          if entity_a == *entity_b {
            continue;
          }

          match (col_a, col_b) {
            | (
              Collider::Circle {
                radius: r_a
              },
              Collider::Rect {
                size: s_b
              }
            ) => {
              let half_b = *s_b * 0.5;
              let closest = Vec2::new(
                pos_a.x.clamp(
                  pos_b.x - half_b.x,
                  pos_b.x + half_b.x
                ),
                pos_a.y.clamp(
                  pos_b.y - half_b.y,
                  pos_b.y + half_b.y
                )
              );

              let dist =
                pos_a.distance(closest);
              if dist < *r_a {
                let normal =
                  if dist > 0.0 {
                    (pos_a - closest)
                      / dist
                  } else {
                    Vec2::Y
                  };
                collisions.push((
                  entity_a,
                  closest
                    + normal * (*r_a),
                  normal
                ));
              }
            }
            | _ => {}
          }
        }
      }
    }

    // 2. Resolve collisions
    let mut query = set.p0();
    for (entity, new_pos, normal) in
      collisions
    {
      if let Ok((
        _,
        mut body,
        mut tf,
        _
      )) = query.get_mut(entity)
      {
        tf.translation = new_pos
          .extend(tf.translation.z);

        // Reflect velocity across
        // normal
        let dot =
          body.velocity.dot(normal);
        if dot < 0.0 {
          body.velocity = (body
            .velocity
            - 2.0 * dot * normal)
            * body.restitution;
        }
      }
    }
  }
}
