use std::path::PathBuf;
use std::sync::OnceLock;

use bevy::prelude::*;
use bevy_egui::{
  EguiContexts,
  EguiPlugin
};
use flatfekt_config::RootConfig;
use flatfekt_runtime::{
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

fn main() -> anyhow::Result<()> {
  let config_path =
    std::env::var_os("FLATFEKT_CONFIG")
      .map(PathBuf::from)
      .unwrap_or_else(
        default_config_path
      );

  let cfg = load_config_or_fail_fast(
    &config_path
  )?;
  init_tracing(&cfg)?;
  enforce_platform_and_render_defaults(
    &cfg
  )?;
  info!(?cfg, "loaded config");

  let scene_path = cfg
    .app
    .as_ref()
    .and_then(|a| a.scene_path.clone())
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

  let mut app =
    build_app(cfg.clone(), scene_file)?;

  if cfg.feature_ui_egui_enabled() {
    info!("enabling egui control UI");
    app
      .add_plugins(EguiPlugin::default())
      .add_systems(
        Update,
        egui_control_panel
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
  mut clock: ResMut<TimelineClock>
) {
  let Ok(ctx) = egui.ctx_mut() else {
    return;
  };
  bevy_egui::egui::Window::new(
    "flatfekt"
  )
  .resizable(true)
  .show(&*ctx, |ui| {
    ui.horizontal(|ui| {
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
        clock.playing = !clock.playing;
      }

      if ui.button("Step").clicked() {
        clock.step_once = true;
      }

      if ui.button("Reset").clicked() {
        clock.t_secs = 0.0;
        clock.accumulator_secs = 0.0;
        clock.playing = false;
      }
    });

    let dur = clock
      .duration_secs
      .unwrap_or(0.0);
    ui.label(format!(
      "t = {:.3}s  dt = {:.6}s  \
       duration = {}",
      clock.t_secs,
      clock.dt_secs,
      if clock.duration_secs.is_some() {
        format!("{dur:.3}s")
      } else {
        "none".to_owned()
      }
    ));

    ui.label(format!(
      "enabled = {}  \
       max_catchup_steps = {}",
      clock.enabled,
      clock.max_catchup_steps
    ));
  });
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
        app:      None,
        logging:  None,
        platform: None,
        render:   None,
        assets:   None,
        features: None,
        runtime:  None
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
  cfg: &RootConfig
) -> anyhow::Result<()> {
  if cfg.unix_backend() != "wayland" {
    anyhow::bail!(
      "unsupported unix backend {:?}; \
       this project defaults to \
       Wayland",
      cfg.unix_backend()
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
      "wayland"
    );
    std::env::set_var(
      "WGPU_BACKEND",
      "vulkan"
    );
  }

  require_vulkan_adapter()
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
