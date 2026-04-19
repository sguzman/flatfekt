#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf
};

use anyhow::Result;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_camera::ScalingMode;
use bevy_mesh::Mesh2d;
use flatfekt_assets::resolve::{
  AssetCache,
  AssetResolveError,
  assets_root,
  bevy_load
};
use flatfekt_config::{
  ConfigError,
  RootConfig
};
use flatfekt_schema::{
  ColorRgba,
  SceneError,
  SceneFile,
  Transform2d
};
use serde::Serialize;
use tracing::instrument;

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
)]
pub struct RenderSortKey {
  pub layer:   u32,
  pub z_bits:  u32,
  pub kind:    u8,
  pub id_hash: u64
}

pub fn render_sort_key(
  id: &str,
  z: f32,
  kind: RenderKind
) -> RenderSortKey {
  RenderSortKey {
    layer:   0,
    z_bits:  z.to_bits(),
    kind:    kind as u8,
    id_hash: fxhash64(id.as_bytes())
  }
}

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum RenderKind {
  Shape  = 0,
  Sprite = 1,
  Text   = 2
}

fn fxhash64(bytes: &[u8]) -> u64 {
  // Simple stable hash for tie-break
  // only (not crypto).
  let mut hash: u64 =
    0xcbf29ce484222325;
  for b in bytes {
    hash ^= *b as u64;
    hash =
      hash.wrapping_mul(0x100000001b3);
  }
  hash
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
  #[error(
    "asset resolution failed: {0}"
  )]
  Assets(#[from] AssetResolveError),

  #[error("scene error: {0}")]
  Scene(#[from] SceneError)
}

#[derive(Resource, Clone)]
pub struct ConfigRes(pub RootConfig);

#[derive(
  Resource, Default, Debug, Clone,
)]
pub struct DebugSettings {
  pub wireframe:   bool,
  pub draw_bounds: bool
}

#[derive(Resource, Clone)]
pub struct SceneRes(pub SceneFile);

#[derive(Resource, Clone)]
pub struct ScenePathRes(pub PathBuf);

#[derive(Resource)]
pub struct SceneFileWatcher {
  pub receiver:
    crossbeam_channel::Receiver<
      notify::Event
    >,
  pub _watcher:
    notify::RecommendedWatcher
}

#[derive(Resource, Clone)]
pub struct AssetsRootRes(pub PathBuf);

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum LoopMode {
  Stop,
  Loop
}

#[derive(Resource, Debug, Clone)]
pub struct TimelineClock {
  pub enabled:           bool,
  pub playing:           bool,
  pub t_secs:            f32,
  pub dt_secs:           f32,
  pub max_catchup_steps: u32,
  pub accumulator_secs:  f32,
  pub duration_secs:     Option<f32>,
  pub loop_mode:         LoopMode,
  pub step_once:         bool
}

impl Default for TimelineClock {
  fn default() -> Self {
    Self {
      enabled:           false,
      playing:           true,
      t_secs:            0.0,
      dt_secs:           1.0 / 60.0,
      max_catchup_steps: 4,
      accumulator_secs:  0.0,
      duration_secs:     None,
      loop_mode:         LoopMode::Stop,
      step_once:         false
    }
  }
}

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  SystemSet,
)]
pub enum FlatfektSet {
  Load,
  Instantiate,
  SimTick
}

#[derive(Resource, Default)]
pub struct SpawnedEntities(
  pub Vec<Entity>
);

#[derive(Resource, Default)]
pub struct EntityMap(
  pub HashMap<String, Vec<Entity>>
);

#[derive(
  Message, bevy::prelude::Event, Default,
)]
pub struct ResetScene;

#[derive(Resource, Default)]
pub struct AssetsCacheRes(
  pub AssetCache
);

pub mod agents;
pub mod animation;
pub mod bake;
pub mod export;
pub mod headless_timeline;
pub mod interaction;
pub mod rapier_backend;
pub mod render_effects;
pub mod simulation;

pub struct FlatfektRuntimePlugin;

impl Plugin for FlatfektRuntimePlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    #[cfg(feature = "physics_rapier2d")]
    {
      let backend = app
        .world()
        .get_resource::<ConfigRes>()
        .map(|c| {
          c.0.simulation_backend()
        })
        .unwrap_or("native");
      if backend == "rapier2d" {
        app.add_plugins(
          rapier_backend::FlatfektRapier2dPlugin
        );
      }
    }

    app.add_message::<ResetScene>()
      .add_message::<ApplyPatch>()
      .add_message::<TransitionScene>()
      .add_message::<SnapshotScene>()
      .add_message::<RequestScreenshot>()
      .add_message::<SeekTimeline>()
      .add_message::<simulation::SimTick>()
      .configure_sets(Startup, FlatfektSet::Instantiate)
      .configure_sets(
        Update,
        (
          FlatfektSet::Load,
          FlatfektSet::SimTick,
          FlatfektSet::Instantiate
        )
      )
      .init_resource::<SpawnedEntities>()
      .init_resource::<EntityMap>()
      .init_resource::<AssetsCacheRes>()
      .init_resource::<TimelineClock>()
      .init_resource::<simulation::SimulationClock>()
      .init_resource::<simulation::SimulationSeed>()
      .init_resource::<simulation::SimRegionRes>()
      .add_systems(
        Startup,
        init_timeline_clock
      )
      .add_systems(
        Startup,
        animation::init_timeline_plan
          .after(init_timeline_clock)
      )
      .add_systems(
        Startup,
        bake::init_baked_simulation
      )
      .add_systems(
        Startup,
        instantiate_scene.in_set(FlatfektSet::Instantiate)
      )
      .init_resource::<animation::TimelinePlan>()
      .add_systems(
        Update,
        animation::seek_timeline_system
          .in_set(FlatfektSet::SimTick)
          .before(timeline_driver)
      )
      .add_systems(
        Update,
        timeline_driver.in_set(FlatfektSet::SimTick)
      )
      .add_systems(
        Update,
        animation::process_timeline_events
          .in_set(FlatfektSet::SimTick)
          .after(timeline_driver)
      )
      .add_systems(
        Update,
        bake::replay_baked_system
          .in_set(FlatfektSet::SimTick)
          .after(timeline_driver)
      )
      .add_systems(
        Startup,
        simulation::init_simulation
          .after(init_timeline_clock)
      )
      .add_systems(
        Update,
        simulation::simulation_driver
          .in_set(FlatfektSet::SimTick)
          .after(timeline_driver)
          .before(animation::process_timeline_events)
      )
      .add_systems(
        Update,
        (
          simulation::gravity_system,
          simulation::bounds_collision_system,
          simulation::entity_collision_system,
        )
          .in_set(FlatfektSet::SimTick)
          .after(simulation::simulation_driver)
          .run_if(|cfg: Res<ConfigRes>| {
            cfg.0.simulation_backend() == "native"
          })
      )
      .add_observer(simulation::sim_control_system)
      .insert_resource(interaction::ActionMap::default())
      .init_resource::<DebugSettings>()
      .add_systems(
        Update,
        (
          simulation::draw_wireframe_system
            .run_if(|cfg: Res<ConfigRes>| {
              cfg.0.simulation_backend() == "native"
            }),
          simulation::draw_physics_debug_system,
          interaction::input_system,
          interaction::picking_system,
          agents::agent_tick_system,
          agents::scripting_system,
        )
      )
      .add_systems(
        Update,
        (
          animation::update_tweens
            .in_set(FlatfektSet::SimTick)
            .after(animation::process_timeline_events),
          animation::update_typewriter
            .in_set(FlatfektSet::SimTick)
            .after(animation::process_timeline_events)
        )
      )
      .add_systems(
        Update,
        (
          apply_patch_system,
          scene_transition_system,
          screenshot_system,
          snapshot_scene_system
        )
          .in_set(FlatfektSet::SimTick)
          .after(animation::process_timeline_events)
      )
      .add_systems(
        Update,
        (reset_scene_system, instantiate_scene)
          .chain()
          .run_if(bevy::ecs::schedule::common_conditions::on_message::<ResetScene>)
          .in_set(FlatfektSet::Instantiate)
      )
      .add_systems(
        Update,
        hot_reload_system.in_set(FlatfektSet::Load)
      );
  }
}

pub fn build_app(
  cfg: RootConfig,
  scene_path: PathBuf,
  scene: SceneFile
) -> Result<App, RuntimeError> {
  let root = assets_root(&cfg)?;
  let mut app = App::new();

  if cfg.feature_hot_reload_enabled() {
    let (tx, rx) =
      crossbeam_channel::unbounded();
    let mut watcher =
      notify::recommended_watcher(
        move |res| {
          match res {
            | Ok(event) => {
              let _ = tx.send(event);
            }
            | Err(err) => {
              tracing::warn!(
                "hot reload watcher \
                 error: {}",
                err
              );
            }
          }
        }
      )
      .map_err(|err| {
        tracing::error!(
          "failed to initialize hot \
           reload watcher: {}",
          err
        );
        RuntimeError::Scene(
          SceneError::Validate(
            format!(
              "failed to initialize \
               hot reload watcher: \
               {err}"
            )
          )
        )
      })?;
    use notify::Watcher;
    let mut watch_paths: Vec<PathBuf> =
      vec![scene_path.clone()];
    if cfg.render_effects_enabled() {
      if let Some(effects) =
        &scene.scene.effects
      {
        for e in effects {
          if let Ok(abs) =
            flatfekt_assets::resolve::resolve_asset_path_cfg(
              &cfg,
              &root,
              &e.wgsl,
            )
          {
            watch_paths.push(abs);
          }
        }
      }
    }

    let mut ok_any = false;
    for p in &watch_paths {
      if let Err(err) = watcher.watch(
        p,
        notify::RecursiveMode::NonRecursive
      ) {
        tracing::error!(
          path = %p.display(),
          "failed to watch path for hot reload: {}",
          err
        );
      } else {
        ok_any = true;
      }
    }

    if !ok_any {
      tracing::error!(
        "hot reload requested but no \
         paths could be watched"
      );
    } else {
      app.insert_resource(
        SceneFileWatcher {
          receiver: rx,
          _watcher: watcher
        }
      );
    }
  }

  app
    .insert_resource(ConfigRes(cfg))
    .insert_resource(ScenePathRes(
      scene_path
    ))
    .insert_resource(SceneRes(scene))
    .insert_resource(AssetsRootRes(
      root
    ))
    .add_plugins(FlatfektRuntimePlugin);

  // Apps in this workspace initialize a
  // `tracing_subscriber` early. Bevy's
  // `LogPlugin` also attempts to
  // install a global logger/subscriber,
  // which causes noisy errors (and can
  // be fatal depending on build
  // settings). We intentionally disable
  // it and rely on our tracing setup.
  let mut plugins =
    DefaultPlugins.build();
  plugins = plugins
    .disable::<bevy::log::LogPlugin>();
  app.add_plugins(plugins);

  if app
    .world()
    .resource::<ConfigRes>()
    .0
    .render_effects_enabled()
  {
    app.add_plugins(
      render_effects::FlatfektEffectsPlugin
    );
  }
  Ok(app)
}

pub fn run_bevy(
  cfg: RootConfig,
  scene_path: PathBuf,
  scene: SceneFile
) -> Result<(), RuntimeError> {
  build_app(cfg, scene_path, scene)?
    .run();
  Ok(())
}

#[instrument(level = "info", skip_all)]
pub(crate) fn init_timeline_clock(
  cfg: Res<ConfigRes>,
  scene: Res<SceneRes>,
  mut clock: ResMut<TimelineClock>
) {
  let cfg = &cfg.0;
  let pb =
    scene.0.scene.playback.as_ref();
  let scene_has_bake =
    scene.0.scene.baked.is_some();
  let scene_has_timeline = scene
    .0
    .scene
    .timeline
    .as_ref()
    .is_some_and(|t| !t.is_empty());
  clock.enabled = cfg
    .runtime_timeline_enabled_opt()
    .unwrap_or(scene_has_timeline)
    || (scene_has_bake
      && cfg
        .runtime_playback_baked_requires_timeline_clock());
  clock.dt_secs = cfg
    .runtime_timeline_fixed_dt_secs();
  clock.max_catchup_steps = cfg
    .runtime_timeline_max_catchup_steps(
    );
  clock.duration_secs =
    pb.and_then(|p| p.duration_secs);
  clock.loop_mode =
    match pb.and_then(|p| {
      p.loop_mode.as_deref()
    }) {
      | Some("loop") => LoopMode::Loop,
      | _ => LoopMode::Stop
    };
  clock.playing = true;
  clock.t_secs = 0.0;
  clock.accumulator_secs = 0.0;
  clock.step_once = false;
  tracing::info!(
    ?clock,
    "initialized timeline clock"
  );
}

#[instrument(level = "debug", skip_all)]
fn timeline_driver(
  time: Res<Time>,
  cfg: Res<ConfigRes>,
  mut clock: ResMut<TimelineClock>
) {
  if !clock.enabled {
    return;
  }

  tracing::trace!(
    t_secs = clock.t_secs,
    playing = clock.playing,
    dt_secs = clock.dt_secs,
    accumulator_secs =
      clock.accumulator_secs,
    "timeline tick begin"
  );

  if clock.step_once {
    clock.step_once = false;
    clock.t_secs += clock.dt_secs;
    enforce_duration(&mut clock);
    tracing::debug!(
      t_secs = clock.t_secs,
      "timeline step_once advanced"
    );
    return;
  }

  if !clock.playing {
    return;
  }

  let dt = if cfg
    .0
    .runtime_timeline_deterministic()
  {
    clock.dt_secs
  } else {
    time.delta_secs()
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
    steps += 1;
    enforce_duration(&mut clock);
    if !clock.playing {
      break;
    }
  }

  if steps > 0 {
    tracing::debug!(
      steps,
      t_secs = clock.t_secs,
      accumulator_secs =
        clock.accumulator_secs,
      "timeline advanced"
    );
  }

  if steps == clock.max_catchup_steps
    && clock.accumulator_secs
      >= clock.dt_secs
  {
    tracing::warn!(
      accumulator_secs =
        clock.accumulator_secs,
      "timeline catch-up cap hit; \
       dropping accumulated time"
    );
    clock.accumulator_secs = 0.0;
  }
}

fn enforce_duration(
  clock: &mut TimelineClock
) {
  let Some(dur) = clock.duration_secs
  else {
    return;
  };
  if !dur.is_finite() || dur <= 0.0 {
    return;
  }
  if clock.t_secs < dur {
    return;
  }

  match clock.loop_mode {
    | LoopMode::Loop => {
      clock.t_secs = clock.t_secs % dur;
    }
    | LoopMode::Stop => {
      clock.t_secs = dur;
      clock.playing = false;
    }
  }
}

#[instrument(level = "info", skip_all)]
fn instantiate_scene(
  mut commands: Commands,
  assets: Res<AssetServer>,
  cfg: Res<ConfigRes>,
  scene: Res<SceneRes>,
  assets_root: Res<AssetsRootRes>,
  effects_rtt: Option<
    Res<render_effects::RenderToTextureRes>
  >,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<
    Assets<ColorMaterial>
  >,
  mut asset_cache: ResMut<
    AssetsCacheRes
  >,
  mut spawned: ResMut<SpawnedEntities>,
  mut entity_map: ResMut<EntityMap>
) {
  let _ = &cfg.0;
  let scene = &scene.0.scene;

  tracing::info!(
    entities = scene.entities.len(),
    has_timeline = scene
      .timeline
      .as_ref()
      .is_some_and(|t| !t.is_empty()),
    "instantiating scene"
  );

  spawned.0.clear();
  entity_map.0.clear();

  let mut camera =
    commands.spawn(Camera2d);
  if let Some(rtt) = effects_rtt {
    camera.insert(Camera {
      order: 0,
      ..default()
    });
    camera.insert(
      bevy_camera::RenderTarget::Image(
        rtt.image.clone().into()
      )
    );
  }
  if let Some(c) = &scene.camera {
    let x = c.x.unwrap_or(0.0);
    let y = c.y.unwrap_or(0.0);
    camera.insert(
      Transform::from_translation(
        Vec3::new(x, y, 0.0)
      )
    );

    let zoom = c.zoom.unwrap_or(1.0);
    let mut ortho = OrthographicProjection::default_2d();
    ortho.scale = 1.0 / zoom;
    if let Some(mode) =
      c.scaling_mode.as_deref()
    {
      ortho.scaling_mode =
        scaling_mode_from_str(mode);
    }
    camera.insert(
      Projection::Orthographic(ortho)
    );
  }

  let clear_color = scene
    .background
    .as_ref()
    .and_then(|b| b.clear_color)
    .or_else(|| {
      scene
        .camera
        .as_ref()
        .and_then(|c| c.clear_color)
    })
    .map(color_from_rgba);

  if let Some(cc) = clear_color {
    commands
      .insert_resource(ClearColor(cc));
  }

  let mut plan: Vec<SpawnOp> =
    Vec::new();
  for ent in &scene.entities {
    let tf = transform_from_spec(
      ent.transform
    );
    let physics = ent.physics.as_ref();
    let collider =
      ent.collider.as_ref();

    if let Some(shape) = &ent.shape {
      plan.push(SpawnOp {
        id: &ent.id,
        kind: RenderKind::Shape,
        tf,
        shape: Some(shape),
        sprite: None,
        text: None,
        physics,
        collider
      });
    }
    if let Some(sprite) = &ent.sprite {
      plan.push(SpawnOp {
        id: &ent.id,
        kind: RenderKind::Sprite,
        tf,
        shape: None,
        sprite: Some(sprite),
        text: None,
        physics,
        collider
      });
    }
    if let Some(text) = &ent.text {
      plan.push(SpawnOp {
        id: &ent.id,
        kind: RenderKind::Text,
        tf,
        shape: None,
        sprite: None,
        text: Some(text),
        physics,
        collider
      });
    }
  }

  plan.sort_by_key(|op| {
    let z = op.tf.translation.z;
    (
      0u32,
      z.to_bits(),
      op.kind as u8,
      op.id
    )
  });

  for op in plan {
    let e = match op.kind {
      | RenderKind::Shape => {
        spawn_shape(
          &mut commands,
          &mut meshes,
          &mut materials,
          &cfg.0,
          op.id,
          scene.defaults.as_ref(),
          op.shape.unwrap(),
          op.tf,
          op.physics,
          op.collider
        )
      }
      | RenderKind::Sprite => {
        spawn_sprite(
          &mut commands,
          &assets,
          &cfg.0,
          &assets_root.0,
          &mut asset_cache.0,
          op.id,
          scene.defaults.as_ref(),
          op.sprite.unwrap(),
          op.tf,
          op.physics,
          op.collider
        )
      }
      | RenderKind::Text => {
        Some(spawn_text(
          &mut commands,
          &assets,
          &cfg.0,
          &assets_root.0,
          &mut asset_cache.0,
          scene.defaults.as_ref(),
          op.text.unwrap(),
          op.tf
        ))
      }
    };

    if let Some(e) = e {
      spawned.0.push(e);
      entity_map
        .0
        .entry(op.id.to_owned())
        .or_default()
        .push(e);
    }
  }
}

struct SpawnOp<'a> {
  id:       &'a str,
  kind:     RenderKind,
  tf:       Transform,
  shape: Option<
    &'a flatfekt_schema::ShapeSpec
  >,
  sprite: Option<
    &'a flatfekt_schema::SpriteSpec
  >,
  text: Option<
    &'a flatfekt_schema::TextSpec
  >,
  physics: Option<
    &'a flatfekt_schema::PhysicsSpec
  >,
  collider: Option<
    &'a flatfekt_schema::ColliderSpec
  >
}

pub(crate) fn transform_from_spec(
  t: Option<Transform2d>
) -> Transform {
  let mut tf = Transform::default();
  if let Some(t) = t {
    tf.translation = Vec3::new(
      t.x,
      t.y,
      t.z.unwrap_or(0.0)
    );
    if let Some(r) = t.rotation {
      tf.rotation =
        Quat::from_rotation_z(r);
    }
    if let Some(s) = t.scale {
      tf.scale = Vec3::splat(s);
    }
  }
  tf
}

fn spawn_sprite(
  commands: &mut Commands,
  assets: &AssetServer,
  cfg: &flatfekt_config::RootConfig,
  assets_root: &PathBuf,
  asset_cache: &mut AssetCache,
  entity_id: &str,
  defaults: Option<
    &flatfekt_schema::DefaultsSpec
  >,
  spec: &flatfekt_schema::SpriteSpec,
  tf: Transform,
  physics: Option<
    &flatfekt_schema::PhysicsSpec
  >,
  collider: Option<
    &flatfekt_schema::ColliderSpec
  >
) -> Option<Entity> {
  let handle =
    bevy_load::load_image_cached(
      asset_cache,
      assets,
      cfg,
      assets_root,
      &spec.image
    );

  match handle {
    | Ok(handle) => {
      let mut sprite =
        Sprite::from_image(handle);
      if let (Some(w), Some(h)) =
        (spec.width, spec.height)
      {
        sprite.custom_size =
          Some(Vec2::new(w, h));
      }
      if let Some(tint) = spec.tint {
        sprite.color =
          color_from_rgba(tint);
      }
      if let Some(opacity) =
        spec.opacity
      {
        let mut srgba =
          sprite.color.to_srgba();
        srgba.alpha = opacity;
        sprite.color =
          Color::Srgba(srgba);
      }
      let mut entity =
        commands.spawn((sprite, tf));
      insert_physics(
        &mut entity,
        cfg,
        physics,
        collider
      );

      if let Some(anchor) = spec
        .anchor
        .as_deref()
        .or_else(|| {
          defaults.and_then(|d| {
            d.sprite_anchor.as_deref()
          })
        })
      {
        entity.insert(anchor_from_str(
          anchor
        ));
      }
      Some(entity.id())
    }
    | Err(err) => {
      tracing::error!(error = %err, id = %entity_id, "failed to load sprite image");
      None
    }
  }
}

fn spawn_text(
  commands: &mut Commands,
  assets: &AssetServer,
  cfg: &flatfekt_config::RootConfig,
  assets_root: &PathBuf,
  asset_cache: &mut AssetCache,
  defaults: Option<
    &flatfekt_schema::DefaultsSpec
  >,
  spec: &flatfekt_schema::TextSpec,
  tf: Transform
) -> Entity {
  tracing::debug!(
    text_value = spec.value.as_deref(),
    text_size = spec.size,
    text_align = spec.align.as_deref(),
    text_anchor =
      spec.anchor.as_deref(),
    "spawning text"
  );
  let font_handle =
    spec.font.as_ref().and_then(|a| {
      bevy_load::load_font_cached(
        asset_cache,
        assets,
        cfg,
        assets_root,
        a
      )
      .ok()
    });

  let mut text_font = TextFont {
    font_size: spec
      .size
      .or_else(|| {
        defaults.and_then(|d| {
          d.text_font_size
        })
      })
      .unwrap_or(24.0),
    ..default()
  };
  if let Some(h) = font_handle {
    text_font.font = h;
  }

  let align = spec
    .align
    .as_deref()
    .or_else(|| {
      defaults.and_then(|d| {
        d.text_align.as_deref()
      })
    });
  let justify = match align {
    | Some("left") => Justify::Left,
    | Some("right") => Justify::Right,
    | _ => Justify::Center
  };

  let text_value =
    if let Some(v) = &spec.value {
      v.clone()
    } else if let Some(spans) =
      &spec.spans
    {
      spans
        .iter()
        .map(|s| s.value.as_str())
        .collect::<String>()
    } else {
      String::new()
    };

  let mut entity = commands.spawn((
    Text2d::new(text_value),
    text_font,
    TextLayout::new_with_justify(
      justify
    ),
    tf
  ));

  if let Some(c) = spec.color {
    entity.insert(TextColor(
      color_from_rgba(c)
    ));
  } else if let Some(c) =
    defaults.and_then(|d| d.text_color)
  {
    entity.insert(TextColor(
      color_from_rgba(c)
    ));
  }

  if let Some(anchor) = spec
    .anchor
    .as_deref()
    .or_else(|| {
      defaults.and_then(|d| {
        d.text_anchor.as_deref()
      })
    })
  {
    entity
      .insert(anchor_from_str(anchor));
  }

  if let Some(effects) = &spec.effects {
    for effect in effects {
      tracing::info!(
        kind = %effect.kind,
        "Applying text effect"
      );
    }
  }

  entity.id()
}

fn spawn_shape(
  commands: &mut Commands,
  meshes: &mut Assets<Mesh>,
  mats: &mut Assets<ColorMaterial>,
  cfg: &flatfekt_config::RootConfig,
  entity_id: &str,
  defaults: Option<
    &flatfekt_schema::DefaultsSpec
  >,
  spec: &flatfekt_schema::ShapeSpec,
  tf: Transform,
  physics: Option<
    &flatfekt_schema::PhysicsSpec
  >,
  collider: Option<
    &flatfekt_schema::ColliderSpec
  >
) -> Option<Entity> {
  let color = spec
    .color
    .map(color_from_rgba)
    .or_else(|| {
      defaults
        .and_then(|d| d.text_color)
        .map(color_from_rgba)
    })
    .unwrap_or(Color::WHITE);

  match spec.kind.as_str() {
    | "rect" => {
      let w = spec.width?;
      let h = spec.height?;
      let sprite = Sprite::from_color(
        color,
        Vec2::new(w, h)
      );
      let mut entity =
        commands.spawn((sprite, tf));
      insert_physics(
        &mut entity,
        cfg,
        physics,
        collider
      );
      Some(entity.id())
    }
    | "circle" => {
      let r = spec.radius?;
      let mesh = meshes.add(
        Mesh::from(Circle::new(r))
      );
      let mat = mats.add(color);
      let mut entity =
        commands.spawn((
          Mesh2d(mesh),
          bevy::prelude::MeshMaterial2d(
            mat
          ),
          tf,
          Name::new(format!(
            "{entity_id}-circle"
          ))
        ));
      insert_physics(
        &mut entity,
        cfg,
        physics,
        collider
      );
      Some(entity.id())
    }
    | "polygon" => {
      let r = spec.radius?;
      let sides = spec.sides?;
      let mesh =
        meshes.add(Mesh::from(
          RegularPolygon::new(r, sides)
        ));
      let mat = mats.add(color);
      let mut entity =
        commands.spawn((
          Mesh2d(mesh),
          bevy::prelude::MeshMaterial2d(
            mat
          ),
          tf,
          Name::new(format!(
            "{entity_id}-polygon"
          ))
        ));
      insert_physics(
        &mut entity,
        cfg,
        physics,
        collider
      );
      Some(entity.id())
    }
    | _ => {
      tracing::error!(id = %entity_id, kind = %spec.kind, "unknown shape kind");
      None
    }
  }
}

pub(crate) fn insert_physics(
  entity: &mut EntityCommands,
  _cfg: &flatfekt_config::RootConfig,
  physics: Option<
    &flatfekt_schema::PhysicsSpec
  >,
  collider: Option<
    &flatfekt_schema::ColliderSpec
  >
) {
  if let Some(p) = physics {
    entity.insert(
      simulation::PhysicsBody {
        velocity:    Vec2::ZERO,
        mass:        p
          .mass
          .unwrap_or(1.0),
        restitution: p
          .restitution
          .unwrap_or(0.5),
        friction:    p
          .friction
          .unwrap_or(0.5),
        fixed:       p.body_type
          == "fixed"
      }
    );
  }
  if let Some(c) = collider {
    match c.kind.as_str() {
      | "circle" => {
        entity.insert(
          simulation::Collider::Circle {
            radius: c.radius.unwrap_or(1.0)
          }
        );
      }
      | "rect" => {
        let size = c
          .size
          .map(Vec2::from)
          .unwrap_or(Vec2::ONE);
        entity.insert(
          simulation::Collider::Rect {
            size
          }
        );
      }
      | _ => {}
    }
  }

  #[cfg(feature = "physics_rapier2d")]
  if _cfg.simulation_backend()
    == "rapier2d"
  {
    use bevy_rapier2d::prelude as r;

    if let Some(p) = physics {
      let rb = if p.body_type == "fixed"
      {
        r::RigidBody::Fixed
      } else {
        r::RigidBody::Dynamic
      };
      entity.insert(rb);

      if let Some(mass) = p.mass {
        if mass.is_finite()
          && mass > 0.0
        {
          entity.insert(r::AdditionalMassProperties::Mass(
            mass,
          ));
        }
      }
    }

    if let Some(c) = collider {
      match c.kind.as_str() {
        | "circle" => {
          let radius =
            c.radius.unwrap_or(1.0);
          entity.insert(
            r::Collider::ball(radius)
          );
        }
        | "rect" => {
          let size = c
            .size
            .map(Vec2::from)
            .unwrap_or(Vec2::ONE);
          entity.insert(
            r::Collider::cuboid(
              size.x * 0.5,
              size.y * 0.5
            )
          );
        }
        | _ => {}
      }
    }

    if let Some(p) = physics {
      entity.insert(
        r::Restitution::coefficient(
          p.restitution.unwrap_or(0.5)
        )
      );
      entity.insert(
        r::Friction::coefficient(
          p.friction.unwrap_or(0.5)
        )
      );
    }
  }
}

pub(crate) fn color_from_rgba(
  c: ColorRgba
) -> Color {
  Color::srgba(
    c.r,
    c.g,
    c.b,
    c.a.unwrap_or(1.0)
  )
}

fn anchor_from_str(a: &str) -> Anchor {
  match a {
    | "top_left" => Anchor::TOP_LEFT,
    | "top" => Anchor::TOP_CENTER,
    | "top_right" => Anchor::TOP_RIGHT,
    | "left" => Anchor::CENTER_LEFT,
    | "right" => Anchor::CENTER_RIGHT,
    | "bottom_left" => {
      Anchor::BOTTOM_LEFT
    }
    | "bottom" => Anchor::BOTTOM_CENTER,
    | "bottom_right" => {
      Anchor::BOTTOM_RIGHT
    }
    | _ => Anchor::CENTER
  }
}

fn scaling_mode_from_str(
  m: &str
) -> ScalingMode {
  match m {
    | "fixed_horizontal" => {
      ScalingMode::FixedHorizontal {
        viewport_width: 1080.0
      }
    }
    | "fixed_vertical" => {
      ScalingMode::FixedVertical {
        viewport_height: 720.0
      }
    }
    | "fixed" => {
      ScalingMode::Fixed {
        width:  1280.0,
        height: 720.0
      }
    }
    | _ => ScalingMode::WindowSize
  }
}

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
  #[error("unknown entity id: {0}")]
  UnknownId(String),

  #[error(
    "entity id has no runtime \
     entities: {0}"
  )]
  EmptyId(String)
}

pub fn lookup_entities<'a>(
  map: &'a EntityMap,
  id: &str
) -> Result<&'a [Entity], LookupError> {
  let Some(list) = map.0.get(id) else {
    tracing::warn!(
      id,
      "entity id lookup failed"
    );
    return Err(LookupError::UnknownId(
      id.to_owned()
    ));
  };
  if list.is_empty() {
    tracing::warn!(
      id,
      "entity id resolved to empty \
       list"
    );
    return Err(LookupError::EmptyId(
      id.to_owned()
    ));
  }
  Ok(list.as_slice())
}

#[instrument(level = "info", skip_all)]
fn reset_scene_system(
  mut commands: Commands,
  spawned: Res<SpawnedEntities>
) {
  for e in &spawned.0 {
    commands.entity(*e).despawn();
  }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
  #[error(
    "failed to load config at {path}: \
     {source}"
  )]
  Config {
    path:   PathBuf,
    #[source]
    source: ConfigError
  },

  #[error(
    "failed to load scene at {path}: \
     {source}"
  )]
  Scene {
    path:   PathBuf,
    #[source]
    source: SceneError
  }
}

#[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
pub fn load_config(
  path: impl AsRef<Path>
) -> Result<RootConfig, LoadError> {
  let path = path.as_ref();
  flatfekt_config::RootConfig::load_from_path(path).map_err(|source| {
    LoadError::Config { path: path.to_path_buf(), source }
  })
}

#[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
pub fn load_scene(
  path: impl AsRef<Path>
) -> Result<SceneFile, LoadError> {
  let path = path.as_ref();
  flatfekt_schema::SceneFile::load_from_path(path).map_err(|source| {
    LoadError::Scene { path: path.to_path_buf(), source }
  })
}

#[derive(Resource, Default)]
pub struct ReloadStatus {
  pub last_error: Option<String>
}

#[instrument(level = "info", skip_all)]
pub fn hot_reload_system(
  watcher: Option<
    Res<SceneFileWatcher>
  >,
  scene_path: Option<Res<ScenePathRes>>,
  cfg: Option<Res<ConfigRes>>,
  mut scene_res: ResMut<SceneRes>,
  assets_root: Option<
    Res<AssetsRootRes>
  >,
  asset_server: Res<AssetServer>,
  mut reload_status: Local<
    ReloadStatus
  >,
  mut last_changed: Local<
    Option<std::time::Instant>
  >,
  mut commands: Commands
) {
  let Some(w) = watcher else {
    return
  };
  let Some(p) = scene_path else {
    return
  };

  let cfg = cfg.as_deref();
  let debounce_scene_ms = cfg
    .map(|c| {
      c.0
        .runtime_hot_reload_debounce_ms(
        )
    })
    .unwrap_or(250);
  let debounce_assets_ms = cfg
    .map(|c| {
      c.0
        .assets_hot_reload_debounce_ms()
    })
    .unwrap_or(250);
  let debounce_ms = debounce_scene_ms
    .min(debounce_assets_ms);
  let warn_and_continue = cfg
    .map(|c| {
      c.0.runtime_hot_reload_warn_and_continue()
    })
    .unwrap_or(true);
  let assets_warn_and_continue = cfg
    .map(|c| {
      c.0
        .assets_hot_reload_warn_and_continue()
    })
    .unwrap_or(true);
  let assets_enabled = cfg
    .map(|c| {
      c.0.assets_hot_reload_enabled()
    })
    .unwrap_or(false);

  let mut changed = false;
  let mut any_wgsl = false;
  let mut changed_paths: Vec<PathBuf> =
    Vec::new();
  for event in w.receiver.try_iter() {
    match event.kind {
      | notify::EventKind::Modify(
        _
      )
      | notify::EventKind::Create(
        _
      ) => {
        changed = true;
      }
      | _ => {}
    }
    for p in &event.paths {
      changed_paths.push(p.clone());
      if p
        .extension()
        .and_then(|e| e.to_str())
        == Some("wgsl")
      {
        any_wgsl = true;
      }
    }
  }

  if changed {
    *last_changed =
      Some(std::time::Instant::now());
  }

  let Some(last) = *last_changed else {
    return;
  };
  if last.elapsed()
    < std::time::Duration::from_millis(
      debounce_ms
    )
  {
    return;
  }
  *last_changed = None;

  {
    let any_scene_toml = changed_paths
      .iter()
      .any(|cp| cp == &p.0);

    if any_scene_toml {
      tracing::info!(
        path = %p.0.display(),
        "scene file change detected; reloading"
      );
    } else if !changed_paths.is_empty()
    {
      tracing::info!(
        paths = changed_paths.len(),
        "non-scene change detected"
      );
    }
    if any_wgsl {
      // Reload shader assets. The
      // AssetServer expects
      // asset-relative paths, so we
      // strip the assets root.
      if let (Some(cfg), Some(ar)) = (
        cfg.as_deref(),
        assets_root.as_deref()
      ) {
        if let Some(scene_effects) =
          &scene_res.0.scene.effects
        {
          for e in scene_effects {
            if let Ok(abs) =
              flatfekt_assets::resolve::resolve_asset_path_cfg(
                &cfg.0,
                &ar.0,
                &e.wgsl,
              )
            {
              let rel = abs
                .strip_prefix(&ar.0)
                .unwrap_or(&abs)
                .to_string_lossy()
                .to_string();
              asset_server.reload(rel);
            }
          }
        }
      } else {
        tracing::warn!(
          "shader change detected but \
           config/assets root \
           unavailable; skipping \
           shader reload"
        );
      }
      reload_status.last_error = None;
      tracing::info!(
        "Shader reload requested"
      );
      return;
    }

    if !any_scene_toml && assets_enabled
    {
      // Reload any assets under the
      // assets root. This is a policy
      // layer on top
      // of Bevy's asset system: we only
      // reload assets we were
      // explicitly asked
      // to watch.
      if let Some(ar) =
        assets_root.as_deref()
      {
        let mut reloaded: usize = 0;
        for abs in &changed_paths {
          if let Ok(rel) =
            abs.strip_prefix(&ar.0)
          {
            let rel = rel
              .to_string_lossy()
              .to_string();
            asset_server.reload(rel);
            reloaded += 1;
          }
        }
        tracing::info!(
          reloaded,
          "asset reload requested"
        );
      } else if assets_warn_and_continue
      {
        tracing::warn!(
          "asset change detected but \
           assets root unavailable; \
           skipping reload"
        );
      } else {
        tracing::error!(
          "asset change detected but \
           assets root unavailable; \
           disabling watcher"
        );
        commands
          .remove_resource::<SceneFileWatcher>();
      }
      return;
    }

    match SceneFile::load_from_path(
      &p.0
    ) {
      | Ok(new_scene) => {
        scene_res.0 = new_scene;
        reload_status.last_error = None;
        commands
          .write_message(ResetScene);
        tracing::info!(
          "Hot-reload successful"
        );
      }
      | Err(err) => {
        reload_status.last_error =
          Some(err.to_string());
        if warn_and_continue {
          tracing::warn!(
            "Hot-reload failed \
             (continuing): {}",
            err
          );
        } else {
          tracing::error!(
            "Hot-reload failed \
             (exiting): {}",
            err
          );
          commands
            .remove_resource::<SceneFileWatcher>();
        }
      }
    }
  }
}

use flatfekt_schema::ScenePatch;

#[derive(
  Message,
  bevy::prelude::Event,
  Clone,
  Debug,
)]
pub struct ApplyPatch(pub ScenePatch);

#[instrument(level = "info", skip_all)]
pub fn apply_patch_system(
  mut events: MessageReader<ApplyPatch>,
  mut scene_res: ResMut<SceneRes>,
  mut commands: Commands
) {
  let mut changed = false;
  let scene = &mut scene_res.0.scene;

  for ev in events.read() {
    changed = true;
    tracing::info!(patch = ?ev.0, "applying patch");
    match &ev.0 {
      | ScenePatch::Add {
        entity
      } => {
        scene
          .entities
          .push(entity.clone());
      }
      | ScenePatch::Remove {
        entity_id
      } => {
        scene.entities.retain(|e| {
          e.id != *entity_id
        });
      }
      | ScenePatch::Update {
        entity_id,
        patch
      } => {
        if let Some(ent) = scene
          .entities
          .iter_mut()
          .find(|e| e.id == *entity_id)
        {
          tracing::debug!(
            entity_id =
              entity_id.as_str(),
            "patch update matched \
             entity"
          );
          if let Some(tags) =
            &patch.tags
          {
            ent.tags =
              Some(tags.clone());
          }
          if let Some(tf) =
            &patch.transform
          {
            ent.transform =
              Some(tf.clone());
          }
          if let Some(sprite) =
            &patch.sprite
          {
            ent.sprite =
              Some(sprite.clone());
          }
          if let Some(text) =
            &patch.text
          {
            ent.text =
              Some(text.clone());
          }
          if let Some(shape) =
            &patch.shape
          {
            ent.shape =
              Some(shape.clone());
          }
        } else {
          tracing::warn!(
            entity_id =
              entity_id.as_str(),
            "patch update target \
             entity not found"
          );
        }
      }
    }
  }

  if changed {
    commands.write_message(ResetScene);
  }
}

#[derive(
  Message,
  bevy::prelude::Event,
  Clone,
  Debug,
)]
pub struct TransitionScene(pub PathBuf);

#[instrument(level = "info", skip_all)]
pub fn scene_transition_system(
  mut events: MessageReader<
    TransitionScene
  >,
  mut scene_res: ResMut<SceneRes>,
  mut reload_status: Local<
    ReloadStatus
  >,
  mut commands: Commands
) {
  let mut do_reset = false;
  for ev in events.read() {
    let p = &ev.0;
    tracing::info!(
      "Transitioning to scene at {:?}",
      p
    );
    match SceneFile::load_from_path(p) {
      | Ok(new_scene) => {
        scene_res.0 = new_scene;
        reload_status.last_error = None;
        do_reset = true;
        tracing::info!(
          "Transition successful"
        );
      }
      | Err(err) => {
        reload_status.last_error =
          Some(err.to_string());
        tracing::error!(
          "Scene transition failed: {}",
          err
        );
      }
    }
  }

  if do_reset {
    commands.write_message(ResetScene);
  }
}

#[derive(
  Message,
  bevy::prelude::Event,
  Clone,
  Debug,
)]
pub struct SnapshotScene;

#[derive(Debug, Clone, Serialize)]
pub struct SceneStateSnapshot {
  pub t_secs:     f32,
  pub playing:    bool,
  pub transforms:
    std::collections::BTreeMap<
      String,
      Transform2d
    >
}

#[derive(
  Message,
  bevy::prelude::Event,
  Clone,
  Debug,
)]
pub struct RequestScreenshot;

#[instrument(level = "info", skip_all)]
pub fn screenshot_system(
  mut events: MessageReader<
    RequestScreenshot
  >,
  mut commands: Commands
) {
  use bevy::render::view::screenshot::{
    save_to_disk,
    Screenshot
  };

  for _ in events.read() {
    let dir = std::path::PathBuf::from(
      ".cache"
    )
    .join("flatfekt")
    .join("screenshots");
    if let Err(e) =
      std::fs::create_dir_all(&dir)
    {
      tracing::error!(
        "failed to create screenshot \
         dir: {}",
        e
      );
      continue;
    }
    let ts =
      std::time::SystemTime::now()
        .duration_since(
          std::time::UNIX_EPOCH
        )
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir
      .join(format!("shot-{ts}.png"));
    tracing::info!(
      path = %path.display(),
      "capturing screenshot"
    );
    commands
      .spawn(
        Screenshot::primary_window()
      )
      .observe(save_to_disk(path));
  }
}

#[derive(
  Message,
  bevy::prelude::Event,
  Clone,
  Debug,
)]
pub struct SeekTimeline {
  pub t_secs:  f32,
  pub playing: bool
}

#[instrument(level = "info", skip_all)]
pub fn snapshot_scene_system(
  mut events: MessageReader<
    SnapshotScene
  >,
  scene_res: Res<SceneRes>,
  scene_path: Res<ScenePathRes>,
  clock: Res<TimelineClock>,
  entity_map: Res<EntityMap>,
  q_transform: Query<&Transform>
) {
  for _ in events.read() {
    let scene_name = scene_path
      .0
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("unknown");
    let snapshot_dir =
      std::path::PathBuf::from(
        ".cache"
      )
      .join("flatfekt")
      .join("scene")
      .join(scene_name);

    if let Err(e) =
      std::fs::create_dir_all(
        &snapshot_dir
      )
    {
      tracing::error!(
        "Failed to create snapshot \
         directory: {}",
        e
      );
      continue;
    }

    let scene_snapshot_path =
      snapshot_dir.join("scene.toml");
    match toml::to_string_pretty(
      &scene_res.0.scene
    ) {
      | Ok(s) => {
        if let Err(e) = std::fs::write(
          &scene_snapshot_path,
          s
        ) {
          tracing::error!(
            "Failed to write \
             snapshot: {}",
            e
          );
        } else {
          tracing::info!(
            "Scene spec snapshot \
             saved to {:?}",
            scene_snapshot_path
          );
        }
      }
      | Err(e) => {
        tracing::error!(
          "Failed to serialize scene \
           snapshot: {}",
          e
        )
      }
    }

    let mut transforms =
      std::collections::BTreeMap::new();
    for (id, list) in &entity_map.0 {
      let Some(first) = list.first()
      else {
        continue;
      };
      if let Ok(tf) =
        q_transform.get(*first)
      {
        transforms.insert(
          id.clone(),
          Transform2d {
            x:        tf.translation.x,
            y:        tf.translation.y,
            rotation: None,
            scale:    Some(tf.scale.x),
            z:        Some(
              tf.translation.z
            )
          }
        );
      }
    }
    let state = SceneStateSnapshot {
      t_secs: clock.t_secs,
      playing: clock.playing,
      transforms
    };

    let state_path =
      snapshot_dir.join("state.toml");
    match toml::to_string_pretty(&state)
    {
      | Ok(s) => {
        if let Err(e) =
          std::fs::write(&state_path, s)
        {
          tracing::error!(
            "Failed to write state \
             snapshot: {}",
            e
          );
        } else {
          tracing::info!(
            "Scene state snapshot \
             saved to {:?}",
            state_path
          );
        }
      }
      | Err(e) => {
        tracing::error!(
          "Failed to serialize state \
           snapshot: {}",
          e
        )
      }
    }
  }
}

pub fn run_bake(
  cfg: flatfekt_config::RootConfig,
  scene_path: std::path::PathBuf,
  scene_file: flatfekt_schema::SceneFile,
  output_path: std::path::PathBuf,
  duration: f32,
  fps: f32
) -> anyhow::Result<()> {
  use bevy::app::ScheduleRunnerPlugin;

  let fps =
    if fps.is_finite() && fps > 0.0 {
      fps
    } else {
      60.0
    };

  // Baking is a simulation export. We
  // intentionally force deterministic
  // fixed stepping and ensure the
  // simulation is enabled regardless of
  // the viewer config.
  let mut cfg = cfg;
  {
    let sim =
      cfg.simulation.get_or_insert_with(
        flatfekt_config::SimulationConfig::default,
      );
    sim.enabled = Some(true);
    sim.playing = Some(true);
    sim.deterministic = Some(true);
    sim.fixed_dt_secs = Some(1.0 / fps);
    sim.max_catchup_steps =
      sim.max_catchup_steps.or(Some(4));
    sim.time_scale =
      sim.time_scale.or(Some(1.0));
  }

  let root = flatfekt_assets::resolve::assets_root(&cfg)
    .unwrap_or_else(|_| std::path::PathBuf::from("assets"));

  let mut app = App::new();
  app.add_plugins(MinimalPlugins.set(
    ScheduleRunnerPlugin::run_loop(
      std::time::Duration::from_secs_f32(
        1.0 / fps
      )
    )
  ));
  app.add_plugins(
    bevy::transform::TransformPlugin
  );

  app.insert_resource(ConfigRes(cfg));
  app.insert_resource(ScenePathRes(
    scene_path
  ));
  app.insert_resource(SceneRes(
    scene_file
  ));
  app.insert_resource(AssetsRootRes(
    root
  ));

  // Messages used by simulation
  // stepping.
  app
    .add_message::<simulation::SimTick>(
    );

  // Minimal scene + simulation stack
  // (avoid window/input systems).
  app
    .configure_sets(Startup, FlatfektSet::Instantiate)
    .configure_sets(Update, FlatfektSet::SimTick)
    .init_resource::<SpawnedEntities>()
    .init_resource::<EntityMap>()
    .init_resource::<simulation::SimulationClock>()
    .init_resource::<simulation::SimulationSeed>()
    .init_resource::<simulation::SimRegionRes>()
    .add_systems(
      Startup,
      simulation::init_simulation,
    )
    .add_systems(
      Startup,
      bake::instantiate_scene_headless_for_bake
        .in_set(FlatfektSet::Instantiate),
    )
    .add_systems(
      Update,
      simulation::simulation_driver.in_set(FlatfektSet::SimTick),
    )
    .add_systems(
      Update,
      (
        simulation::gravity_system,
        simulation::bounds_collision_system,
        simulation::entity_collision_system,
      )
        .in_set(FlatfektSet::SimTick)
        .after(simulation::simulation_driver),
    );

  app.init_resource::<bake::BakeRecorder>();
  app.add_systems(
    Update,
    bake::bake_recording_system
      .after(simulation::entity_collision_system)
  );

  app.add_systems(
    Update,
    move |clock: Res<
      simulation::SimulationClock
    >,
          recorder: Res<
      bake::BakeRecorder
    >,
          mut exit: MessageWriter<
      bevy::app::AppExit
    >| {
      if clock.t_secs >= duration {
        bake::save_bake(
          &recorder,
          &output_path
        )
        .unwrap();
        exit.write(
          bevy::app::AppExit::Success
        );
      }
    }
  );

  app.run();
  Ok(())
}
