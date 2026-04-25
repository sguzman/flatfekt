use std::path::{
  Path,
  PathBuf
};

use anyhow::Context;
use bevy::prelude::*;
use serde::{
  Deserialize,
  Serialize
};
use tracing::instrument;

use crate::{
  SeekTimeline,
  TimelineClock
};

pub const EXPORT_FRAMES_DIR: &str =
  "frames";
pub const EXPORT_MANIFEST_JSON: &str =
  "export.json";
pub const EXPORT_FRAME_PATTERN: &str =
  "frame-%06d.png";

#[derive(Debug, Clone)]
pub struct ExportPaths {
  pub dir:        PathBuf,
  pub frames_dir: PathBuf,
  pub manifest:   PathBuf
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

pub fn export_output_dir_for_scene(
  output_root: &Path,
  scene_path: &Path,
  scene_xxhash64_hex: &str,
  created_unix_secs: u64,
  pid: u32
) -> PathBuf {
  let scene_stem = scene_path
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("scene");
  output_root
    .join(sanitize_component(
      scene_stem
    ))
    .join("exports")
    .join(scene_xxhash64_hex)
    .join(format!(
      "run-{}-{}",
      created_unix_secs, pid
    ))
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ExportManifest {
  pub version:               String,
  pub created_unix_secs:     u64,
  pub tool:                  String,
  pub tool_version:          String,
  pub source_scene_path:     String,
  pub source_scene_xxhash64: String,
  pub fps:                   f32,
  pub duration_secs:         f32,
  pub frame_count:           u64,
  pub frame_pattern:         String
}

#[derive(Resource, Debug, Clone)]
pub struct ExportFramesJob {
  pub output_dir:        PathBuf,
  pub frames_dir:        PathBuf,
  pub manifest_path:     PathBuf,
  pub fps:               f32,
  pub duration_secs:     f32,
  pub frame_count:       u64,
  pub width:             u32,
  pub height:            u32,
  pub overwrite:         bool,
  pub window_visible:    bool,
  pub next_frame_index:  u64,
  pub pending_frame_png:
    Option<PathBuf>,
  pub manifest:          ExportManifest
}

impl ExportFramesJob {
  pub fn new(
    paths: ExportPaths,
    fps: f32,
    duration_secs: f32,
    width: u32,
    height: u32,
    overwrite: bool,
    window_visible: bool,
    manifest: ExportManifest
  ) -> anyhow::Result<Self> {
    anyhow::ensure!(
      fps.is_finite() && fps > 0.0,
      "fps must be > 0"
    );
    anyhow::ensure!(
      duration_secs.is_finite()
        && duration_secs > 0.0,
      "duration_secs must be > 0"
    );
    let frame_count =
      (duration_secs * fps)
        .round()
        .max(1.0) as u64;
    Ok(Self {
      output_dir: paths.dir,
      frames_dir: paths.frames_dir,
      manifest_path: paths.manifest,
      fps,
      duration_secs,
      frame_count,
      width,
      height,
      overwrite,
      window_visible,
      next_frame_index: 0,
      pending_frame_png: None,
      manifest
    })
  }
}

#[derive(Default)]
pub struct FlatfektExportPlugin;

impl Plugin for FlatfektExportPlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    app.add_systems(
      Startup,
      (
        export_prepare_system,
        export_configure_window_system,
      )
        .chain()
    )
    .add_systems(Update, export_frames_system);
  }
}

#[instrument(level = "info", skip_all)]
fn export_prepare_system(
  job: Option<ResMut<ExportFramesJob>>,
  mut exit: MessageWriter<
    bevy::app::AppExit
  >
) {
  let Some(mut job) = job else {
    return;
  };

  if job.output_dir.exists() {
    if !job.overwrite {
      tracing::error!(
        dir = %job.output_dir.display(),
        "export output directory exists; refusing to overwrite"
      );
      exit.write(
        bevy::app::AppExit::error()
      );
      return;
    }
  }

  if let Err(err) =
    std::fs::create_dir_all(
      &job.frames_dir
    )
  {
    tracing::error!(
      dir = %job.frames_dir.display(),
      "failed to create frames dir: {}",
      err
    );
    exit.write(
      bevy::app::AppExit::error()
    );
    return;
  }

  job.manifest.frame_count =
    job.frame_count;
  job.manifest.fps = job.fps;
  job.manifest.duration_secs =
    job.duration_secs;

  match serde_json::to_vec_pretty(
    &job.manifest
  ) {
    | Ok(bytes) => {
      if let Err(err) = std::fs::write(
        &job.manifest_path,
        bytes
      ) {
        tracing::error!(
          path = %job.manifest_path.display(),
          "failed to write export manifest: {}",
          err
        );
        exit.write(
          bevy::app::AppExit::error()
        );
        return;
      }
    }
    | Err(err) => {
      tracing::error!(
        "failed to serialize export \
         manifest: {}",
        err
      );
      exit.write(
        bevy::app::AppExit::error()
      );
      return;
    }
  }

  tracing::info!(
    dir = %job.output_dir.display(),
    frame_count = job.frame_count,
    fps = job.fps,
    duration_secs = job.duration_secs,
    "export frames job prepared"
  );
}

#[instrument(level = "info", skip_all)]
fn export_configure_window_system(
  mut job: Option<
    ResMut<ExportFramesJob>
  >,
  mut windows: Query<&mut Window>
) {
  let Some(job) = job.as_deref_mut()
  else {
    return;
  };
  for mut w in &mut windows {
    w.visible = job.window_visible;
    w.resolution.set(
      job.width as f32,
      job.height as f32
    );
  }
}

#[instrument(level = "debug", skip_all)]
fn export_frames_system(
  job: Option<ResMut<ExportFramesJob>>,
  mut clock: ResMut<TimelineClock>,
  mut seek: MessageWriter<SeekTimeline>,
  mut commands: Commands,
  mut exit: MessageWriter<
    bevy::app::AppExit
  >
) {
  use bevy::render::view::screenshot::{
    save_to_disk,
    Screenshot
  };

  let Some(mut job) = job else {
    return;
  };

  if let Some(pending) =
    job.pending_frame_png.clone()
  {
    if pending.exists() {
      job.pending_frame_png = None;
      job.next_frame_index += 1;

      if job.next_frame_index % 60 == 0
        || job.next_frame_index
          == job.frame_count
      {
        tracing::debug!(
          done = job.next_frame_index,
          total = job.frame_count,
          "export progress"
        );
      }
    } else {
      return;
    }
  }

  if job.next_frame_index
    >= job.frame_count
  {
    tracing::info!(
      dir = %job.output_dir.display(),
      "export frames complete"
    );
    exit.write(
      bevy::app::AppExit::Success
    );
    return;
  }

  let frame_index =
    job.next_frame_index;
  let t_secs =
    frame_index as f32 / job.fps;

  clock.enabled = true;
  clock.playing = false;
  clock.t_secs = t_secs;
  seek.write(SeekTimeline {
    t_secs,
    playing: false
  });

  let png =
    job.frames_dir.join(format!(
      "frame-{frame_index:06}.png"
    ));

  commands
    .spawn(Screenshot::primary_window())
    .observe(save_to_disk(png.clone()));

  job.pending_frame_png = Some(png);
}

pub fn export_paths_from_output_dir(
  dir: PathBuf
) -> ExportPaths {
  let frames_dir =
    dir.join(EXPORT_FRAMES_DIR);
  let manifest =
    dir.join(EXPORT_MANIFEST_JSON);
  ExportPaths {
    dir,
    frames_dir,
    manifest
  }
}

#[derive(Debug, Clone)]
pub struct FfmpegMp4Options {
  pub overwrite: bool,
  pub codec:     String,
  pub preset:    String,
  pub crf:       u32,
  pub pix_fmt:   String
}

pub fn ffmpeg_mp4_args(
  frames_dir: &Path,
  fps: f32,
  out_mp4: &Path,
  opts: &FfmpegMp4Options
) -> Vec<std::ffi::OsString> {
  let input_pattern = frames_dir
    .join(EXPORT_FRAME_PATTERN);
  let mut args: Vec<
    std::ffi::OsString
  > = Vec::with_capacity(32);

  if opts.overwrite {
    args.push("-y".into());
  } else {
    args.push("-n".into());
  }
  args.push("-hide_banner".into());
  args.push("-loglevel".into());
  args.push("error".into());
  args.push("-framerate".into());
  args.push(format!("{fps:.6}").into());
  args.push("-start_number".into());
  args.push("0".into());
  args.push("-i".into());
  args.push(input_pattern.into());
  args.push("-c:v".into());
  args.push(opts.codec.clone().into());
  args.push("-preset".into());
  args.push(opts.preset.clone().into());
  args.push("-crf".into());
  args
    .push(opts.crf.to_string().into());
  args.push("-pix_fmt".into());
  args
    .push(opts.pix_fmt.clone().into());
  args.push("-movflags".into());
  args.push("+faststart".into());
  args.push(out_mp4.into());

  args
}

#[instrument(level = "info", skip_all)]
pub fn now_unix_secs()
-> anyhow::Result<u64> {
  use std::time::{
    SystemTime,
    UNIX_EPOCH
  };
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .context(
      "system clock before unix epoch"
    )?;
  Ok(now.as_secs())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn export_dir_sanitizes_scene_stem() {
    let out =
      export_output_dir_for_scene(
        Path::new(
          ".cache/flatfekt/scene"
        ),
        Path::new(
          "scenes/weird name.toml"
        ),
        "deadbeef",
        123,
        456
      );
    assert!(
      out
        .to_string_lossy()
        .contains("weird_name"),
      "path should sanitize stem: {}",
      out.display()
    );
    assert!(
      out
        .to_string_lossy()
        .contains("/exports/"),
      "path should include exports: {}",
      out.display()
    );
  }

  #[test]
  fn ffmpeg_args_include_start_number_zero()
   {
    let opts = FfmpegMp4Options {
      overwrite: true,
      codec:     "libx264".to_owned(),
      preset:    "medium".to_owned(),
      crf:       18,
      pix_fmt:   "yuv420p".to_owned()
    };
    let args = ffmpeg_mp4_args(
      Path::new(".cache/frames"),
      60.0,
      Path::new("out.mp4"),
      &opts
    );
    let joined = args
      .iter()
      .map(|s| s.to_string_lossy())
      .collect::<Vec<_>>()
      .join(" ");
    assert!(
      joined
        .contains("-start_number 0"),
      "args should include \
       start_number 0: {joined}"
    );
  }
}
