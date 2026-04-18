#![cfg(feature = "physics_rapier2d")]

use bevy::prelude::*;
use bevy_rapier2d::prelude as r;
use tracing::instrument;

use crate::{
  ConfigRes,
  DebugSettings,
  SceneRes,
  simulation
};

pub struct FlatfektRapier2dPlugin;

impl Plugin for FlatfektRapier2dPlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    app.add_plugins(
      r::RapierPhysicsPlugin::<
        r::NoUserData
      >::default(),
    )
    .add_plugins(
      r::RapierDebugRenderPlugin::default(),
    )
    .init_resource::<r::TimestepMode>()
    .add_systems(
      Startup,
      (
        init_rapier_from_config,
        init_rapier_debug_render,
      )
        .chain(),
    )
    .add_systems(
      Update,
      (
        sync_rapier_debug_render_enabled,
        sync_rapier_pipeline_active,
      ),
    );
  }
}

#[instrument(level = "info", skip_all)]
fn init_rapier_from_config(
  cfg: Res<ConfigRes>,
  scene: Res<SceneRes>,
  mut timestep_mode: ResMut<
    r::TimestepMode
  >,
  mut q_cfg: Query<
    &mut r::RapierConfiguration
  >
) {
  let backend =
    cfg.0.simulation_backend();
  if backend != "rapier2d" {
    return;
  }

  // Gravity: default from our sim
  // region base; allow per-scene
  // override.
  let mut gravity =
    Vec2::new(0.0, -9.81);
  if let Some(sim_spec) =
    &scene.0.scene.simulation
  {
    if let Some(g) = sim_spec.gravity {
      gravity = Vec2::from(g);
    }
  }
  let mut any = false;
  for mut rapier_cfg in q_cfg.iter_mut()
  {
    rapier_cfg.gravity = gravity;
    any = true;
  }
  if !any {
    tracing::warn!(
      "rapier configuration entity \
       not found; gravity not applied"
    );
  }

  let dt =
    cfg.0.simulation_fixed_dt_secs();
  if cfg.0.simulation_deterministic() {
    *timestep_mode =
      r::TimestepMode::Fixed {
        dt,
        substeps: 1
      };
  } else {
    *timestep_mode =
      r::TimestepMode::Variable {
        max_dt:     dt,
        time_scale: 1.0,
        substeps:   1
      };
  }

  tracing::info!(
    dt,
    "initialized rapier2d \
     configuration"
  );
}

#[instrument(level = "info", skip_all)]
fn init_rapier_debug_render(
  mut debug_ctx: ResMut<
    r::DebugRenderContext
  >
) {
  // Default off; UI toggles will enable
  // it.
  debug_ctx.enabled = false;
}

#[instrument(level = "trace", skip_all)]
fn sync_rapier_debug_render_enabled(
  cfg: Res<ConfigRes>,
  settings: Res<DebugSettings>,
  mut debug_ctx: ResMut<
    r::DebugRenderContext
  >
) {
  if cfg.0.simulation_backend()
    != "rapier2d"
  {
    return;
  }
  debug_ctx.enabled =
    settings.wireframe;
}

#[instrument(level = "trace", skip_all)]
fn sync_rapier_pipeline_active(
  cfg: Res<ConfigRes>,
  sim_clock: Res<
    simulation::SimulationClock
  >,
  mut q_cfg: Query<
    &mut r::RapierConfiguration
  >
) {
  if cfg.0.simulation_backend()
    != "rapier2d"
  {
    return;
  }
  let active = sim_clock.enabled
    && sim_clock.playing;
  for mut rapier_cfg in q_cfg.iter_mut()
  {
    rapier_cfg
      .physics_pipeline_active = active;
  }
}
