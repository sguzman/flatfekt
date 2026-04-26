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
  build_app,
  export,
  load_config,
  load_scene
};
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod ui;

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
  /// Export a deterministic PNG frame
  /// sequence (for motion graphics)
  /// from a scene or bake artifact.
  ///
  /// If a scene TOML is provided,
  /// Flatfekt bakes first, then exports
  /// frames from baked playback.
  ExportFrames {
    input:          PathBuf,
    /// Override `export.frames.
    /// output_root`.
    #[arg(long)]
    output_root:    Option<PathBuf>,
    /// Override FPS (frames per
    /// second).
    #[arg(long)]
    fps:            Option<f32>,
    /// Override duration (seconds).
    #[arg(long)]
    duration_secs:  Option<f32>,
    /// Override render width (pixels).
    #[arg(long)]
    width:          Option<u32>,
    /// Override render height
    /// (pixels).
    #[arg(long)]
    height:         Option<u32>,
    /// Overwrite existing export
    /// output directory, if any.
    #[arg(long)]
    overwrite:      bool,
    /// Render with a visible window
    /// (useful for debugging export).
    #[arg(long)]
    window_visible: bool
  },
  /// Export MP4 video from a scene or
  /// bake artifact.
  ///
  /// If a scene TOML is provided,
  /// Flatfekt bakes first, then exports
  /// frames and encodes them via
  /// `ffmpeg`.
  ExportMp4 {
    input:          PathBuf,
    /// Optional output mp4 path.
    #[arg(long)]
    out:            Option<PathBuf>,
    /// Override FPS (frames per
    /// second).
    #[arg(long)]
    fps:            Option<f32>,
    /// Override duration (seconds).
    #[arg(long)]
    duration_secs:  Option<f32>,
    /// Override render width (pixels).
    #[arg(long)]
    width:          Option<u32>,
    /// Override render height
    /// (pixels).
    #[arg(long)]
    height:         Option<u32>,
    /// Overwrite output files, if any.
    #[arg(long)]
    overwrite:      bool,
    /// Keep exported PNG frames after
    /// encoding.
    #[arg(long)]
    keep_frames:    bool,
    /// Render with a visible window
    /// (useful for debugging export).
    #[arg(long)]
    window_visible: bool
  },
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
        scene.clone(),
        scene_file
      )
      .context("build app")?;
      ui::maybe_add_ui_plugins(
        &cfg,
        scene_allows_inspector,
        &mut app
      )?;
      app.run();
      Ok(())
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
        scene_path,
        scene_file
      )
      .context(
        "build baked playback app"
      )?;
      ui::maybe_add_ui_plugins(
        &cfg,
        scene_allows_inspector,
        &mut app
      )?;
      app.run();
      Ok(())
    }
    | Command::ExportFrames {
      input,
      output_root,
      fps,
      duration_secs,
      width,
      height,
      overwrite,
      window_visible
    } => {
      configure_unix_backend_env(
        &cfg, cli.x11
      )?;
      require_vulkan_adapter()?;

      let export_req = ExportRequest {
        input,
        output_root,
        fps,
        duration_secs,
        width,
        height,
        overwrite,
        window_visible,
        keep_frames: true
      };

      let res = run_export_frames(
        cfg, export_req
      )
      .context("export frames")?;
      println!(
        "{}",
        res.output_dir.display()
      );
      eprintln!(
        "frames: {}\nmanifest: \
         {}\nfps: {:.3}\nduration: \
         {:.3}s\nresolution: \
         {}x{}\nframe_count: {}",
        res.frames_dir.display(),
        res.manifest_path.display(),
        res.fps,
        res.duration_secs,
        res.width,
        res.height,
        res.frame_count
      );
      Ok(())
    }
    | Command::ExportMp4 {
      input,
      out,
      fps,
      duration_secs,
      width,
      height,
      overwrite,
      keep_frames,
      window_visible
    } => {
      configure_unix_backend_env(
        &cfg, cli.x11
      )?;
      require_vulkan_adapter()?;

      let export_req = ExportRequest {
        input,
        output_root: None,
        fps,
        duration_secs,
        width,
        height,
        overwrite,
        window_visible,
        keep_frames: keep_frames
      };

      let out = run_export_mp4(
        cfg, export_req, out
      )
      .context("export mp4")?;
      println!("{}", out.display());
      Ok(())
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

#[derive(Debug, Clone)]
struct ExportRequest {
  input:          PathBuf,
  output_root:    Option<PathBuf>,
  fps:            Option<f32>,
  duration_secs:  Option<f32>,
  width:          Option<u32>,
  height:         Option<u32>,
  overwrite:      bool,
  window_visible: bool,
  keep_frames:    bool
}

#[derive(Debug, Clone)]
struct ExportFramesResult {
  output_dir:    PathBuf,
  frames_dir:    PathBuf,
  manifest_path: PathBuf,
  fps:           f32,
  duration_secs: f32,
  width:         u32,
  height:        u32,
  frame_count:   u64
}

fn run_export_frames(
  cfg: RootConfig,
  req: ExportRequest
) -> anyhow::Result<ExportFramesResult>
{
  let (
    bake_dir,
    bake_json_path,
    scene_path,
    cfg
  ) = resolve_export_input(cfg, &req)?;

  tracing::info!(
    bake_dir = %bake_dir.display(),
    bake_json = %bake_json_path.display(),
    scene_playback = %scene_path.display(),
    "export uses baked playback input"
  );

  let baked =
    std::fs::read(&bake_json_path)
      .with_context(|| {
        format!(
          "failed to read bake json \
           at {}",
          bake_json_path.display()
        )
      })?;
  let baked: bake::BakedSimulation =
    serde_json::from_slice(&baked)
      .context(
        "failed to parse bake json"
      )?;

  let created_unix_secs =
    export::now_unix_secs()?;
  let pid = std::process::id();

  let scene_src_path = PathBuf::from(
    &baked.meta.source_scene_path
  );
  let scene_hash = baked
    .meta
    .source_scene_xxhash64
    .clone();

  let output_root = req
    .output_root
    .unwrap_or_else(|| {
      cfg.export_frames_output_root()
    });

  let output_dir =
    export::export_output_dir_for_scene(
      &output_root,
      &scene_src_path,
      &scene_hash,
      created_unix_secs,
      pid
    );
  let paths =
    export::export_paths_from_output_dir(
      output_dir
    );

  let fps =
    req.fps.unwrap_or_else(|| {
      cfg.export_frames_default_fps()
    });
  let duration_secs =
    req.duration_secs.unwrap_or_else(
      || {
        cfg.export_frames_default_duration_secs()
      }
    );

  let width =
    req.width.unwrap_or_else(|| {
      cfg.export_frames_width()
    });
  let height =
    req.height.unwrap_or_else(|| {
      cfg.export_frames_height()
    });
  let frame_count =
    (duration_secs * fps)
      .round()
      .max(1.0) as u64;

  let manifest =
    export::ExportManifest {
      version: "0.1".to_string(),
      created_unix_secs,
      tool: "flatfekt".to_string(),
      tool_version: env!(
        "CARGO_PKG_VERSION"
      )
      .to_string(),
      source_scene_path: baked
        .meta
        .source_scene_path
        .clone(),
      source_scene_xxhash64: baked
        .meta
        .source_scene_xxhash64
        .clone(),
      fps,
      duration_secs,
      frame_count: 0,
      frame_pattern: format!(
        "{}/frame-%06d.png",
        export::EXPORT_FRAMES_DIR
      )
    };

  let job =
    export::ExportFramesJob::new(
      paths.clone(),
      fps,
      duration_secs,
      width,
      height,
      req.overwrite
        || cfg
          .export_frames_overwrite(),
      req.window_visible
        || cfg
          .export_frames_window_visible(
          ),
      manifest
    )
    .context("invalid export job")?;

  let scene_file =
    load_scene(&scene_path)?;
  let mut app = build_app(
    cfg.clone(),
    scene_path.clone(),
    scene_file
  )
  .context("build export app")?;
  app.add_plugins(
    export::FlatfektExportPlugin
  );
  app.insert_resource(job);

  tracing::info!(
    width,
    height,
    "starting export render loop"
  );
  app.run();

  Ok(ExportFramesResult {
    output_dir: paths.dir,
    frames_dir: paths.frames_dir,
    manifest_path: paths.manifest,
    fps,
    duration_secs,
    width,
    height,
    frame_count
  })
}

fn run_export_mp4(
  cfg: RootConfig,
  req: ExportRequest,
  out: Option<PathBuf>
) -> anyhow::Result<PathBuf> {
  let frames = run_export_frames(
    cfg.clone(),
    req.clone()
  )
  .context("export frames for mp4")?;

  let out_mp4 = match out {
    | None => {
      frames
        .output_dir
        .join("video.mp4")
    }
    | Some(p) if p.is_dir() => {
      p.join("video.mp4")
    }
    | Some(p) => p
  };

  let ffmpeg_path =
    cfg.export_video_ffmpeg_path();
  ensure_ffmpeg_available(
    &ffmpeg_path
  )?;

  encode_mp4_from_png_sequence(
    &ffmpeg_path,
    &frames.frames_dir,
    frames.fps,
    &out_mp4,
    req.overwrite
      || cfg.export_video_overwrite(),
    cfg.export_video_codec(),
    cfg.export_video_preset(),
    cfg.export_video_crf(),
    cfg.export_video_pix_fmt()
  )?;

  if !req.keep_frames
    && !cfg.export_video_keep_frames()
  {
    tracing::info!(
      dir = %frames.frames_dir.display(),
      "removing exported frames"
    );
    let _ = std::fs::remove_dir_all(
      &frames.frames_dir
    );
  }

  eprintln!(
    "export_dir: {}\nframes: \
     {}\nmanifest: {}\nmp4: {}",
    frames.output_dir.display(),
    frames.frames_dir.display(),
    frames.manifest_path.display(),
    out_mp4.display()
  );

  Ok(out_mp4)
}

fn resolve_export_input(
  cfg: RootConfig,
  req: &ExportRequest
) -> anyhow::Result<(
  PathBuf,
  PathBuf,
  PathBuf,
  RootConfig
)> {
  if req
    .input
    .extension()
    .and_then(|e| e.to_str())
    == Some("toml")
    && !req.input.is_dir()
  {
    let scene_path = req.input.clone();
    let scene_bytes =
      std::fs::read(&scene_path)
        .with_context(|| {
          format!(
            "failed to read scene \
             bytes at {}",
            scene_path.display()
          )
        })?;
    let scene_file =
      load_scene(&scene_path)?;

    let bake_req = bake::BakeRequest {
      output_root: cfg.export_bake_output_root(),
      fps: req.fps.unwrap_or_else(|| {
        cfg.export_bake_default_fps()
      }),
      duration_secs: req
        .duration_secs
        .unwrap_or_else(|| {
          cfg.export_bake_default_duration_secs()
        }),
      copy_assets: cfg.export_bake_copy_assets(),
    };

    tracing::info!(
      scene = %scene_path.display(),
      copy_assets = bake_req.copy_assets,
      "auto-baking scene for export"
    );
    let bake_out =
      bake::bake_scene_to_dir(
        cfg.clone(),
        scene_path.clone(),
        scene_bytes,
        scene_file,
        bake_req
      )
      .context(
        "auto-bake for export failed"
      )?;

    let bake_dir = bake_out.dir;
    let bake_json_path =
      bake_out.bake_json_path;
    let scene_playback_path =
      bake_out.scene_playback_path;

    let mut cfg = cfg;
    cfg
      .app
      .get_or_insert_with(
        Default::default
      )
      .assets_dir =
      Some(bake_dir.clone());

    return Ok((
      bake_dir,
      bake_json_path,
      scene_playback_path,
      cfg
    ));
  }

  let (bake_dir, bake_json_path) =
    if req.input.is_dir() {
      (
        req.input.clone(),
        req
          .input
          .join(bake::BAKE_JSON_FILE)
      )
    } else {
      let parent = req
        .input
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
          PathBuf::from(".")
        });
      (parent, req.input.clone())
    };

  if !bake_json_path.exists() {
    anyhow::bail!(
      "bake json not found at {}",
      bake_json_path.display()
    );
  }

  let scene_playback_path = bake_dir
    .join(
      bake::BAKE_SCENE_PLAYBACK_TOML
    );
  if !scene_playback_path.exists() {
    anyhow::bail!(
      "scene_playback.toml not found \
       at {}",
      scene_playback_path.display()
    );
  }

  let mut cfg = cfg;
  cfg
    .app
    .get_or_insert_with(
      Default::default
    )
    .assets_dir =
    Some(bake_dir.clone());

  Ok((
    bake_dir,
    bake_json_path,
    scene_playback_path,
    cfg
  ))
}

fn ensure_ffmpeg_available(
  ffmpeg: &PathBuf
) -> anyhow::Result<()> {
  let mut cmd =
    std::process::Command::new(ffmpeg);
  cmd.arg("-version");
  let out = cmd.output().with_context(
    || {
      format!(
        "failed to run ffmpeg at {}",
        ffmpeg.display()
      )
    }
  )?;
  if !out.status.success() {
    anyhow::bail!(
      "ffmpeg failed -version (status \
       {})",
      out.status
    );
  }
  Ok(())
}

#[allow(clippy::too_many_arguments)]
fn encode_mp4_from_png_sequence(
  ffmpeg: &PathBuf,
  frames_dir: &PathBuf,
  fps: f32,
  out_mp4: &PathBuf,
  overwrite: bool,
  codec: String,
  preset: String,
  crf: u32,
  pix_fmt: String
) -> anyhow::Result<()> {
  if out_mp4.exists() && !overwrite {
    anyhow::bail!(
      "output mp4 already exists at \
       {}; pass --overwrite or set \
       export.video.overwrite=true",
      out_mp4.display()
    );
  }

  let mut cmd =
    std::process::Command::new(ffmpeg);
  let args = flatfekt_runtime::export::ffmpeg_mp4_args(
    frames_dir,
    fps,
    out_mp4,
    &flatfekt_runtime::export::FfmpegMp4Options {
      overwrite,
      codec,
      preset,
      crf,
      pix_fmt
    }
  );
  cmd.args(args);

  tracing::info!(
    ffmpeg = %ffmpeg.display(),
    frames = %frames_dir.display(),
    out = %out_mp4.display(),
    "encoding mp4 via ffmpeg"
  );

  let out = cmd.output().with_context(
    || {
      format!(
        "failed to run ffmpeg at {}",
        ffmpeg.display()
      )
    }
  )?;
  if !out.status.success() {
    let stderr =
      String::from_utf8_lossy(
        &out.stderr
      );
    anyhow::bail!(
      "ffmpeg encoding failed (status \
       {}): {}",
      out.status,
      stderr
    );
  }
  Ok(())
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
        ui:         None,
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
