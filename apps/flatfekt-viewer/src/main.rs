use std::path::PathBuf;
use std::sync::OnceLock;

use bevy::prelude::*;
use bevy_egui::{
  EguiContexts,
  EguiPlugin,
  EguiPrimaryContextPass,
  PrimaryEguiContext
};
use clap::Parser;
use flatfekt_config::RootConfig;
use flatfekt_runtime::{
  ConfigRes,
  LoadError,
  TimelineClock,
  build_app,
  load_config,
  load_scene
};
use tracing::{
  info,
  warn
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser, Debug)]
#[command(name = "flatfekt-viewer")]
#[command(about = "GUI viewer for flatfekt scenes", long_about = None)]
struct ViewerCli {
  /// Path to the control-pane config
  /// TOML.
  #[arg(long)]
  config: Option<PathBuf>,

  /// Path to a specific scene TOML to
  /// override the config default.
  #[arg(long)]
  scene: Option<PathBuf>,

  /// Use X11 instead of Wayland on
  /// Unix (Wayland remains the
  /// default).
  #[arg(long)]
  x11: bool
}

fn main() -> anyhow::Result<()> {
  let cli = ViewerCli::parse();

  let config_path = cli
    .config
    .or_else(|| {
      std::env::var_os(
        "FLATFEKT_CONFIG"
      )
      .map(PathBuf::from)
    })
    .unwrap_or_else(
      default_config_path
    );

  let cfg = load_config_or_fail_fast(
    &config_path
  )?;
  init_tracing(&cfg)?;
  enforce_platform_and_render_defaults(
    &cfg, cli.x11
  )?;
  info!(?cfg, "loaded config");

  let scene_path = cli
    .scene
    .or_else(|| {
      cfg.app.as_ref().and_then(|a| {
        a.scene_path.clone()
      })
    })
    .unwrap_or_else(|| {
      PathBuf::from("scenes/demo.toml")
    });

  let scene_file = load_scene(
    &scene_path
  )
  .map_err(|e| anyhow::anyhow!(e))?;
  info!(path = %scene_path.display(), "loaded scene");

  ensure_cache_layout(&scene_path)?;

  let scene_allows_inspector =
    scene_file
      .scene
      .playback
      .as_ref()
      .and_then(|p| {
        p.enable_introspection
      })
      .unwrap_or(false);

  let mut app = build_app(
    cfg.clone(),
    scene_path.clone(),
    scene_file
  )?;

  if cfg.feature_ui_egui_enabled() {
    info!("enabling egui control UI");
    app
      .add_plugins(EguiPlugin::default())
      .add_systems(
        Update,
        ensure_primary_egui_context
          .run_if(no_primary_egui_context),
      )
      .add_systems(
        EguiPrimaryContextPass,
        egui_control_panel,
      );
  }

  if cfg
    .feature_inspector_egui_enabled()
    && scene_allows_inspector
  {
    info!(
      "enabling bevy-inspector-egui"
    );
    app.add_plugins(
      bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
    );
  }

  app.run();

  Ok(())
}

fn egui_control_panel(
  mut egui: EguiContexts,
  mut clock: ResMut<TimelineClock>,
  cfg: Res<ConfigRes>,
  time: Res<Time>,
  mut commands: Commands,
  scene: Res<
    flatfekt_runtime::SceneRes
  >,
  entity_map: Res<
    flatfekt_runtime::EntityMap
  >,
  mut debug_settings: ResMut<
    flatfekt_runtime::DebugSettings
  >,
  query: Query<(
    Entity,
    Option<&Name>,
    Option<&Transform>,
    Option<&Sprite>,
    Option<&Text2d>
  )>,
  mut wheel_seek_step: Local<
    Option<f32>
  >
) {
  let Ok(ctx) = egui.ctx_mut() else {
    return;
  };

  bevy_egui::egui::TopBottomPanel::top(
    "top_panel"
  )
  .show(&*ctx, |ui| {
    ui.horizontal(|ui| {
      ui.label("Flatfekt Scene Viewer");
      ui.separator();
      ui.label(format!(
        "Scene: {}",
        scene
          .0
          .scene
          .camera
          .as_ref()
          .map(|_| "Demo")
          .unwrap_or("Demo")
      ));
      ui.separator();
      ui.label(format!(
        "FPS: {:.1}",
        1.0
          / time
            .delta_secs()
            .max(0.0001)
      ));
      ui.separator();
      let mode = if clock.enabled {
        "Fixed DT"
      } else {
        "Realtime"
      };
      ui.label(format!(
        "Tick Mode: {mode}"
      ));
    });
  });

  bevy_egui::egui::Window::new(
    "Timeline Controls"
  )
  .resizable(true)
  .show(&*ctx, |ui| {
    let wheel_seek_cfg = &cfg.0;
    let seek_min =
      wheel_seek_cfg.ui_timeline_wheel_seek_secs_min();
    let seek_max =
      wheel_seek_cfg.ui_timeline_wheel_seek_secs_max();
    let ctrl_scale =
      wheel_seek_cfg.ui_timeline_wheel_seek_ctrl_scale();
    let default_step =
      wheel_seek_cfg.ui_timeline_wheel_seek_secs();
    let step = wheel_seek_step
      .get_or_insert(default_step);

    ui.horizontal(|ui| {
      if ui.button("|<").clicked() {
        tracing::info!(
          "UI Action: Seek Start"
        );
        clock.t_secs = 0.0;
        commands.trigger(
          flatfekt_runtime::SeekTimeline {
            t_secs:   0.0,
            playing: clock.playing
          }
        );
      }
      if ui.button("<<").clicked() {
        tracing::info!(
          "UI Action: Rewind"
        );
        let t =
          (clock.t_secs - 1.0).max(0.0);
        clock.t_secs = t;
        commands.trigger(
          flatfekt_runtime::SeekTimeline {
            t_secs:   t,
            playing: clock.playing
          }
        );
      }
      if ui
        .button(
          if clock.playing {
            "Pause"
          } else {
            "Play"
          }
        )
        .clicked()
      {
        tracing::info!(
          "UI Action: Toggle \
           Play/Pause (playing: {})",
          !clock.playing
        );
        clock.playing = !clock.playing;
        commands.trigger(
          flatfekt_runtime::SeekTimeline {
            t_secs:   clock.t_secs,
            playing: clock.playing
          }
        );
      }
      if ui.button("Step").clicked() {
        tracing::info!(
          "UI Action: Step"
        );
        clock.step_once = true;
        clock.playing = false;
        commands.trigger(
          flatfekt_runtime::SeekTimeline {
            t_secs:   clock.t_secs,
            playing: false
          }
        );
      }
      if ui.button(">>").clicked() {
        tracing::info!(
          "UI Action: Fast Forward"
        );
        let t = clock.t_secs + 1.0;
        clock.t_secs =
          clock.duration_secs.map(|d| t.min(d)).unwrap_or(t);
        commands.trigger(
          flatfekt_runtime::SeekTimeline {
            t_secs:   clock.t_secs,
            playing: clock.playing
          }
        );
      }
      if ui.button(">|").clicked() {
        if let Some(d) = clock.duration_secs {
          tracing::info!(
            "UI Action: Seek End"
          );
          clock.t_secs = d;
          commands.trigger(
            flatfekt_runtime::SeekTimeline {
              t_secs:   d,
              playing: clock.playing
            }
          );
        }
      }
      if ui.button("Reset").clicked() {
        tracing::info!(
          "UI Action: Reset Timeline"
        );
        clock.t_secs = 0.0;
        clock.accumulator_secs = 0.0;
        clock.playing = false;
        commands.trigger(
          flatfekt_runtime::SeekTimeline {
            t_secs:   0.0,
            playing: false
          }
        );
      }
      if ui.button("Screenshot").clicked() {
        tracing::info!(
          "UI Action: Screenshot"
        );
        commands.trigger(
          flatfekt_runtime::RequestScreenshot
        );
      }
    });

    let dur = clock.duration_secs.unwrap_or_else(|| {
      clock.t_secs.max(1.0)
    });
    let mut scrub = clock.t_secs;
    let resp = ui.add(
      bevy_egui::egui::Slider::new(
        &mut scrub,
        0.0..=dur
      )
      .text("Scrubber"),
    );

    // Wheel seeking while hovered.
    if resp.hovered() && dur.is_finite() {
      let scroll_y =
        ui.input(|i| i.raw_scroll_delta.y);
      if scroll_y.abs() >= 0.01 {
        let ctrl =
          ui.input(|i| i.modifiers.ctrl);
        let sign = scroll_y.signum();
        if ctrl {
          let scaled = if sign > 0.0 {
            *step * ctrl_scale
          } else {
            *step / ctrl_scale
          };
          *step =
            scaled.clamp(seek_min, seek_max);
        } else {
          let new_t =
            (clock.t_secs + (*step * sign)).clamp(0.0, dur);
          clock.t_secs = new_t;
          commands.trigger(
            flatfekt_runtime::SeekTimeline {
              t_secs:   new_t,
              playing: clock.playing
            }
          );
        }
      }
    }

    if resp.changed() {
      clock.t_secs = scrub;
      commands.trigger(
        flatfekt_runtime::SeekTimeline {
          t_secs:   scrub,
          playing: clock.playing
        }
      );
    }

    ui.horizontal(|ui| {
      ui.label(format!(
        "Time: {:.3}s",
        clock.t_secs
      ));
      ui.separator();
      if let Some(d) =
        clock.duration_secs
      {
        ui.label(format!(
          "Duration: {:.3}s",
          d
        ));
      } else {
        ui.label("Duration: none");
      }
      ui.separator();
      ui.label(format!(
        "Behavior: {:?}",
        clock.loop_mode
      ));
    });
  });

  bevy_egui::egui::Window::new("Entity Inspector").resizable(true).show(&*ctx, |ui| {
    bevy_egui::egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
      for (id, entities) in &entity_map.0 {
        ui.collapsing(format!("ID: {}", id), |ui| {
          for &entity in entities {
            if let Ok((_e, name, tf, sprite, text)) = query.get(entity) {
              ui.label(format!("Entity: {:?}", entity));
              if let Some(n) = name { ui.label(format!("Name: {}", n.as_str())); }
              if let Some(t) = tf { ui.label(format!("Transform: {:.2?}", t.translation)); }
              if sprite.is_some() { ui.label("Type: Sprite"); }
              if text.is_some() { ui.label("Type: Text"); }
            }
          }
        });
      }
    });
  });

  bevy_egui::egui::Window::new(
    "Dev Panels"
  )
  .resizable(true)
  .show(&*ctx, |ui| {
    ui.checkbox(
      &mut debug_settings.wireframe,
      "Wireframe Mode"
    );
    ui.checkbox(
      &mut debug_settings.draw_bounds,
      "Draw World Bounds"
    );
    ui.collapsing(
      "Performance",
      |ui| {
        ui.label(format!(
          "Frame Time: {:.2}ms",
          time.delta_secs() * 1000.0
        ));
        ui.label(format!(
          "Sim Tick Time: {:.2}ms",
          clock.dt_secs * 1000.0
        ));
        ui.label(
          "Asset Load Stats: [Not \
           instrumented]"
        );
      }
    );
    ui.collapsing(
      "Reload Status",
      |ui| {
        ui.label("Status: OK");
        ui.label("Last Error: None");
      }
    );
    ui.collapsing("Help", |ui| {
      ui.label(
        "This is the flatfekt viewer."
      );
      ui.label(
        "- Use the Timeline Controls \
         to scrub through the scene."
      );
      ui.label(
        "- Use the Entity Inspector \
         to view entity details."
      );
    });
  });
}

fn ensure_primary_egui_context(
  mut commands: Commands,
  q: Query<
    Entity,
    (
      With<Camera2d>,
      Without<PrimaryEguiContext>
    )
  >
) {
  if let Some(entity) = q.iter().next()
  {
    commands
      .entity(entity)
      .insert(PrimaryEguiContext);
    tracing::info!(
      ?entity,
      "tagged primary camera with \
       PrimaryEguiContext"
    );
  }
}

fn init_tracing(
  cfg: &RootConfig
) -> anyhow::Result<()> {
  let filter = cfg
    .logging
    .as_ref()
    .and_then(|l| l.filter.as_deref())
    .map(|s| s.to_owned())
    .or_else(|| {
      cfg
        .logging
        .as_ref()
        .and_then(|l| {
          l.level.as_deref()
        })
        .map(|lvl| lvl.to_owned())
    })
    .or_else(|| {
      std::env::var("RUST_LOG").ok()
    })
    .unwrap_or_else(|| {
      "info".to_owned()
    });

  let env_filter =
    tracing_subscriber::EnvFilter::new(
      filter
    );

  let stderr_layer =
    tracing_subscriber::fmt::layer()
      .with_target(true)
      .with_level(true);

  let registry =
    tracing_subscriber::registry()
      .with(env_filter)
      .with(stderr_layer);

  if cfg.app_mode() == "dev" {
    let logs_dir = cache_logs_dir();
    std::fs::create_dir_all(&logs_dir)?;
    let file_name =
      run_log_file_name()?;
    let file_appender =
      tracing_appender::rolling::never(
        logs_dir, file_name
      );
    let (non_blocking, guard) =
      tracing_appender::non_blocking(
        file_appender
      );
    LOG_GUARD.set(guard).map_err(
      |_| {
        anyhow::anyhow!(
          "log guard already \
           initialized"
        )
      }
    )?;

    let file_layer =
      tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_writer(non_blocking);

    registry.with(file_layer).init();
  } else {
    registry.init();
  }

  Ok(())
}

static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

fn load_config_or_fail_fast(
  path: &PathBuf
) -> anyhow::Result<RootConfig> {
  match load_config(path) {
    | Ok(cfg) => Ok(cfg),
    | Err(LoadError::Config {
      ..
    }) if !path.exists() => {
      warn!(path = %path.display(), "config file not found; using built-in defaults");
      Ok(RootConfig {
        app:        None,
        logging:    None,
        platform:   None,
        render:     None,
        assets:     None,
        features:   None,
        runtime:    None,
        export:     None,
        ui:         None,
        simulation: None
      })
    }
    | Err(err) => Err(err.into())
  }
}

fn default_config_path() -> PathBuf {
  let preferred = PathBuf::from(
    ".config/flatfekt/flatfekt.toml"
  );
  if preferred.exists() {
    return preferred;
  }
  PathBuf::from("flatfekt.toml")
}

fn enforce_platform_and_render_defaults(
  cfg: &RootConfig,
  x11: bool
) -> anyhow::Result<()> {
  let configured = cfg.unix_backend();
  let ub = if x11 {
    if configured != "x11" {
      warn!(
        configured,
        "overriding \
         platform.unix_backend to x11 \
         for this run"
      );
    }
    "x11"
  } else {
    configured
  };
  if ub != "wayland" && ub != "x11" {
    anyhow::bail!(
      "unsupported unix backend {:?}; \
       expected \"wayland\" or \"x11\"",
      ub
    );
  }
  if cfg.render_backend() != "vulkan" {
    anyhow::bail!(
      "unsupported render backend \
       {:?}; this project requires \
       Vulkan",
      cfg.render_backend()
    );
  }

  // Rust 2024: mutating process
  // environment is `unsafe` because it
  // can violate invariants when other
  // threads read environment variables
  // concurrently. We do this at startup
  // before spinning up any engine
  // threads.
  unsafe {
    std::env::set_var(
      "WINIT_UNIX_BACKEND",
      ub
    );
    std::env::set_var(
      "WGPU_BACKEND",
      "vulkan"
    );
  }

  preflight_display_env(ub)?;
  require_vulkan_adapter()
}

fn preflight_display_env(
  ub: &str
) -> anyhow::Result<()> {
  match ub {
    | "x11" => {
      if std::env::var_os("DISPLAY")
        .is_none()
      {
        anyhow::bail!(
          "x11 selected but DISPLAY \
           is not set; run under an \
           X11 session (or Xwayland) \
           or set DISPLAY"
        );
      }
    }
    | "wayland" => {
      let has_wayland =
        std::env::var_os(
          "WAYLAND_DISPLAY"
        )
        .is_some()
          || std::env::var_os(
            "WAYLAND_SOCKET"
          )
          .is_some();
      if !has_wayland {
        anyhow::bail!(
          "wayland selected but \
           neither WAYLAND_DISPLAY \
           nor WAYLAND_SOCKET is set; \
           run under a Wayland session"
        );
      }
    }
    | _ => {}
  }
  Ok(())
}

fn require_vulkan_adapter()
-> anyhow::Result<()> {
  let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

  let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;

  let info = adapter.get_info();
  info!(
    ?info,
    "Vulkan adapter selected"
  );
  Ok(())
}

fn cache_root_dir() -> PathBuf {
  PathBuf::from(".cache/flatfekt")
}

fn cache_logs_dir() -> PathBuf {
  cache_root_dir().join("logs")
}

fn cache_scene_dir(
  scene_path: &PathBuf
) -> PathBuf {
  let scene_name = scene_path
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("scene");
  cache_root_dir().join("scene").join(
    sanitize_component(scene_name)
  )
}

fn sanitize_component(
  input: &str
) -> String {
  input
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric()
        || matches!(c, '-' | '_' | '.')
      {
        c
      } else {
        '_'
      }
    })
    .collect()
}

fn ensure_cache_layout(
  scene_path: &PathBuf
) -> anyhow::Result<()> {
  std::fs::create_dir_all(
    cache_root_dir()
  )?;
  std::fs::create_dir_all(
    cache_logs_dir()
  )?;
  std::fs::create_dir_all(
    cache_scene_dir(scene_path)
  )?;
  Ok(())
}

fn run_log_file_name()
-> anyhow::Result<String> {
  use std::time::{
    SystemTime,
    UNIX_EPOCH
  };

  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| {
      anyhow::anyhow!(
        "system clock before unix \
         epoch: {e}"
      )
    })?;
  let pid = std::process::id();
  Ok(format!(
    "run-{}-{}.{:09}.log",
    now.as_secs(),
    pid,
    now.subsec_nanos()
  ))
}

fn no_primary_egui_context(
  q: Query<&PrimaryEguiContext>
) -> bool {
  q.is_empty()
}
