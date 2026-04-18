use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::Context;
use clap::{
  Parser,
  Subcommand
};
use flatfekt_config::RootConfig;
use flatfekt_runtime::{
  LoadError,
  bake,
  load_config,
  load_scene,
  run_bevy
};
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser, Debug)]
#[command(name = "flatfekt")]
#[command(about = "TOML-driven 2D Bevy scene runner", long_about = None)]
struct Cli {
  /// Path to the control-pane config
  /// TOML.
  #[arg(
    long,
    default_value = ".config/flatfekt/\
                     flatfekt.toml"
  )]
  config: PathBuf,

  /// Use X11 instead of Wayland on
  /// Unix (Wayland remains the
  /// default).
  #[arg(long)]
  x11: bool,

  /// Override `logging.level`
  /// (trace|debug|info|warn|error).
  #[arg(long)]
  log_level: Option<String>,

  /// Override `logging.filter`
  /// (tracing filter string).
  #[arg(long)]
  log_filter: Option<String>,

  #[command(subcommand)]
  command: Command
}

#[derive(Subcommand, Debug)]
enum Command {
  /// Validate a scene TOML against the
  /// schema and current control pane.
  Validate { scene: PathBuf },
  /// Run a scene (overrides any
  /// configured `app.scene_path`).
  Run { scene: PathBuf },
  /// Bake a simulation to a first-class
  /// artifact directory containing
  /// `bake.json`,
  /// `scene_playback.toml`,
  /// and packaged assets.
  Bake {
    scene:          PathBuf,
    /// Override `export.bake.
    /// output_root`.
    #[arg(long)]
    output_root:    Option<PathBuf>,
    /// Override FPS for deterministic
    /// bake stepping.
    #[arg(long)]
    fps:            Option<f32>,
    /// Override duration (seconds) for
    /// baking.
    #[arg(long)]
    duration_secs:  Option<f32>,
    /// Do not copy external assets
    /// into the bake directory.
    #[arg(long)]
    no_copy_assets: bool
  },
  /// Run baked playback from a bake
  /// directory (or directly from
  /// `bake.json`), with simulation
  /// disabled.
  PlayBake { input: PathBuf },
  /// Run a scene's timeline headlessly
  /// (no window) and log dispatched
  /// events + patch application.
  TraceTimeline {
    scene:         PathBuf,
    /// Maximum number of fixed-dt
    /// steps to run.
    #[arg(long, default_value_t = 600)]
    max_steps:     u32,
    /// Optional maximum time (seconds)
    /// to run; overrides scene
    /// duration.
    #[arg(long)]
    max_time_secs: Option<f32>
  }
}

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let cfg = load_config_or_fail_fast(
    &cli.config
  )?;
  init_tracing(
    &cfg,
    cli.log_level.as_deref(),
    cli.log_filter.as_deref()
  )?;

  match cli.command {
    | Command::Validate {
      scene
    } => {
      let _scene = load_scene(&scene)
        .map_err(|e| {
        match e {
          | LoadError::Scene {
            ..
          } => e,
          | LoadError::Config {
            ..
          } => e
        }
      })?;
      tracing::info!(path = %scene.display(), "scene valid");
      Ok(())
    }
    | Command::Run {
      scene
    } => {
      configure_unix_backend_env(
        &cfg, cli.x11
      )?;
      require_vulkan_adapter()?;
      let scene_file =
        load_scene(&scene)?;
      run_bevy(cfg, scene, scene_file)
        .context("runtime failed")
    }
    | Command::Bake {
      scene,
      output_root,
      fps,
      duration_secs,
      no_copy_assets
    } => {
      let scene_bytes =
        std::fs::read(&scene)
          .with_context(|| {
            format!(
              "failed to read scene \
               bytes at {}",
              scene.display()
            )
          })?;
      let scene_file =
        load_scene(&scene)?;

      let req = bake::BakeRequest {
        output_root: output_root
          .unwrap_or_else(|| cfg.export_bake_output_root()),
        fps: fps.unwrap_or_else(|| cfg.export_bake_default_fps()),
        duration_secs: duration_secs.unwrap_or_else(|| {
          cfg.export_bake_default_duration_secs()
        }),
        copy_assets: !no_copy_assets
          && cfg.export_bake_copy_assets(),
      };

      let out =
        bake::bake_scene_to_dir(
          cfg,
          scene.clone(),
          scene_bytes,
          scene_file,
          req
        )?;

      println!("{}", out.dir.display());
      tracing::info!(
        dir = %out.dir.display(),
        bake_json = %out.bake_json_path.display(),
        playback_scene = %out.scene_playback_path.display(),
        "bake complete"
      );
      Ok(())
    }
    | Command::PlayBake {
      input
    } => {
      configure_unix_backend_env(
        &cfg, cli.x11
      )?;
      require_vulkan_adapter()?;

      let (bake_dir, bake_json_path) =
        if input.is_dir() {
          (
            input.clone(),
            input.join(
              bake::BAKE_JSON_FILE
            )
          )
        } else {
          let parent = input
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
              PathBuf::from(".")
            });
          (parent, input.clone())
        };

      if !bake_json_path.exists() {
        anyhow::bail!(
          "bake json not found at {}",
          bake_json_path.display()
        );
      }

      let scene_path = bake_dir.join(
        bake::BAKE_SCENE_PLAYBACK_TOML
      );
      if !scene_path.exists() {
        anyhow::bail!(
          "scene_playback.toml not \
           found at {}",
          scene_path.display()
        );
      }

      // Ensure packaged assets resolve
      // relative to the bake directory.
      let mut cfg = cfg;
      cfg
        .app
        .get_or_insert_with(
          Default::default
        )
        .assets_dir = Some(bake_dir);

      let scene_file =
        load_scene(&scene_path)?;
      run_bevy(
        cfg, scene_path, scene_file
      )
      .context(
        "baked playback runtime failed"
      )
    }
    | Command::TraceTimeline {
      scene,
      max_steps,
      max_time_secs
    } => {
      let mut scene_file =
        load_scene(&scene)?;
      let opts =
        flatfekt_runtime::headless_timeline::HeadlessTimelineOptions {
          max_steps,
          max_time_secs
        };
      let res =
        flatfekt_runtime::headless_timeline::run_headless_timeline(
          &cfg,
          &mut scene_file,
          &opts
        )?;
      tracing::info!(
        ?res,
        "headless timeline run \
         complete"
      );
      Ok(())
    }
  }
}

fn load_config_or_fail_fast(
  path: &PathBuf
) -> anyhow::Result<RootConfig> {
  match load_config(path) {
    | Ok(cfg) => Ok(cfg),
    | Err(LoadError::Config {
      ..
    }) if !path.exists() => {
      warn!(path = %path.display(), "config file not found; using built-in defaults");
      let cfg = RootConfig {
        app:        None,
        logging:    None,
        platform:   None,
        render:     None,
        assets:     None,
        features:   None,
        runtime:    None,
        export:     None,
        simulation: None
      };
      cfg.validate().context(
        "default config invalid"
      )?;
      Ok(cfg)
    }
    | Err(e) => Err(anyhow::anyhow!(e))
  }
}

fn init_tracing(
  cfg: &RootConfig,
  level_override: Option<&str>,
  filter_override: Option<&str>
) -> anyhow::Result<()> {
  let filter = filter_override
    .map(|s| s.to_owned())
    .or_else(|| {
      level_override
        .map(|s| s.to_owned())
    })
    .or_else(|| {
      cfg
        .logging
        .as_ref()
        .and_then(|l| {
          l.filter.as_deref()
        })
        .map(|s| s.to_owned())
    })
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

fn cache_logs_dir() -> PathBuf {
  PathBuf::from(".cache")
    .join("flatfekt")
    .join("logs")
}

fn run_log_file_name()
-> anyhow::Result<String> {
  let now =
    std::time::SystemTime::now()
      .duration_since(
        std::time::UNIX_EPOCH
      )
      .context(
        "system time before unix epoch"
      )?;
  Ok(format!(
    "run-{}.log",
    now.as_secs()
  ))
}

fn require_vulkan_adapter()
-> anyhow::Result<()> {
  let instance =
    wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::VULKAN,
      ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

  let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
    power_preference: wgpu::PowerPreference::HighPerformance,
    compatible_surface: None,
    force_fallback_adapter: false,
  }))?;
  let info = adapter.get_info();
  tracing::info!(
    ?info,
    "Vulkan adapter selected"
  );
  Ok(())
}

fn configure_unix_backend_env(
  cfg: &RootConfig,
  x11: bool
) -> anyhow::Result<()> {
  // Rust 2024: mutating process
  // environment is `unsafe` because it
  // can violate invariants when other
  // threads read environment variables
  // concurrently. We do this at startup
  // before spinning up any engine
  // threads.
  unsafe {
    std::env::set_var(
      "WGPU_BACKEND",
      "vulkan"
    );
  }

  let configured = cfg.unix_backend();
  let backend = if x11 {
    if configured != "x11" {
      tracing::warn!(
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

  unsafe {
    std::env::set_var(
      "WINIT_UNIX_BACKEND",
      backend
    );
  }
  tracing::info!(
    backend,
    "unix backend selected"
  );

  preflight_display_env(backend)?;
  Ok(())
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
