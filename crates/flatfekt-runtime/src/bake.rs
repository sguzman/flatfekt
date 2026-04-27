use std::collections::HashMap;
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
use sha2::{
  Digest,
  Sha256
};
use tracing::instrument;
use xxhash_rust::xxh64::xxh64;

use crate::{
  ConfigRes,
  EntityMap,
  ScenePathRes,
  SceneRes,
  SpawnedEntities,
  simulation
};

pub const BAKE_JSON_FILE: &str =
  "bake.json";
pub const BAKE_SCENE_PLAYBACK_TOML:
  &str = "scene_playback.toml";
pub const BAKE_ASSETS_DIR: &str =
  "assets";
pub const BAKE_VERSION: &str = "0.2";

#[derive(Debug, Clone)]
pub struct BakeRequest {
  pub output_root:   PathBuf,
  pub fps:           f32,
  pub duration_secs: f32,
  pub copy_assets:   bool
}

#[derive(Debug, Clone)]
pub struct BakeOutput {
  pub dir:                 PathBuf,
  pub bake_json_path:      PathBuf,
  pub scene_playback_path: PathBuf
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  Resource,
)]
pub struct BakedSimulation {
  pub version:  String,
  pub meta:     BakeMeta,
  pub playback: BakePlayback,
  pub assets:   Vec<BakeAsset>,
  pub entities:
    HashMap<String, BakedEntity>,
  pub events:   Vec<BakedEvent>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakeMeta {
  pub created_unix_secs:     u64,
  pub tool:                  String,
  pub tool_version:          String,
  pub source_scene_path:     String,
  pub source_scene_xxhash64: String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakePlayback {
  pub fps:           f32,
  pub dt_secs:       f32,
  pub duration_secs: f32,
  pub loop_mode:     String,
  pub end_behavior:  String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakeAsset {
  pub role:          String,
  pub original_ref:  String,
  pub packaged_path: String,
  pub sha256:        String,
  pub bytes:         u64
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakedEntity {
  pub keyframes: Vec<BakedKeyframe>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakedKeyframe {
  pub t:           f32,
  pub transform:   BakedTransform,
  pub text_value:  Option<String>,
  pub sprite_rgba: Option<[f32; 4]>
}

#[derive(
  Debug,
  Clone,
  Copy,
  Serialize,
  Deserialize,
)]
pub struct BakedTransform {
  pub x:     f32,
  pub y:     f32,
  pub z:     f32,
  pub r:     f32,
  pub scale: f32
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakedEvent {
  pub t:      f32,
  pub action: String,
  pub target: Option<String>
}

#[derive(Resource)]
pub struct BakeRecorder {
  pub data: BakedSimulation
}

impl Default for BakeRecorder {
  fn default() -> Self {
    Self {
      data: BakedSimulation {
        version:  BAKE_VERSION
          .to_owned(),
        meta:     BakeMeta {
          created_unix_secs:     0,
          tool:
            "flatfekt".to_owned(),
          tool_version:
            "0.0.0".to_owned(),
          source_scene_path:
            String::new(),
          source_scene_xxhash64:
            String::new()
        },
        playback: BakePlayback {
          fps:           60.0,
          dt_secs:       1.0 / 60.0,
          duration_secs: 0.0,
          loop_mode:     "stop"
            .to_owned(),
          end_behavior:  "stop"
            .to_owned()
        },
        assets:   Vec::new(),
        entities: HashMap::new(),
        events:   Vec::new()
      }
    }
  }
}

#[derive(Resource, Debug, Clone)]
pub struct BakeSettings {
  pub bake_json_path:      PathBuf,
  pub scene_playback_path: PathBuf,
  pub source_scene:
    flatfekt_schema::SceneFile,
  pub playback:            BakePlayback,
  pub meta:                BakeMeta,
  pub assets: Vec<BakeAsset>,
  pub assets_root:         PathBuf
}

pub fn xxhash64_hex(
  bytes: &[u8]
) -> String {
  let h = xxh64(bytes, 0);
  format!("{h:016x}")
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

pub fn bake_output_dir_for_scene(
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
    .join("bakes")
    .join(scene_xxhash64_hex)
    .join(format!(
      "run-{}-{}",
      created_unix_secs, pid
    ))
}

fn now_unix_secs() -> anyhow::Result<u64>
{
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

fn sha256_hex(bytes: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bytes);
  let digest = hasher.finalize();
  let mut out =
    String::with_capacity(64);
  for b in digest {
    use std::fmt::Write as _;
    let _ = write!(out, "{b:02x}");
  }
  out
}

fn package_builtin_shaders(
  cfg: &flatfekt_config::RootConfig,
  output_dir: &Path,
  assets_dir: &Path,
  assets: &mut Vec<BakeAsset>
) {
  let Ok(src_assets_root) =
    flatfekt_assets::resolve::assets_root(
      cfg
    )
  else {
    tracing::debug!(
      "assets root not configured; skipping shader directory packaging"
    );
    return;
  };

  let src_shaders_dir = src_assets_root
    .join("flatfekt")
    .join("shaders");
  if !src_shaders_dir.exists()
    || !src_shaders_dir.is_dir()
  {
    return;
  }

  let dst_shaders_in_assets =
    assets_dir
      .join("flatfekt")
      .join("shaders");
  let _ = std::fs::create_dir_all(
    &dst_shaders_in_assets
  );

  let dst_shaders_in_root = output_dir
    .join("flatfekt")
    .join("shaders");
  let _ = std::fs::create_dir_all(
    &dst_shaders_in_root
  );

  let Ok(entries) =
    std::fs::read_dir(&src_shaders_dir)
  else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_file() {
      continue;
    }
    let Some(name) = path.file_name()
    else {
      continue;
    };
    let Ok(bytes) =
      std::fs::read(&path)
    else {
      continue;
    };

    let _ = std::fs::write(
      dst_shaders_in_assets.join(name),
      &bytes
    );
    let _ = std::fs::write(
      dst_shaders_in_root.join(name),
      &bytes
    );

    let sha256 = sha256_hex(&bytes);
    assets.push(BakeAsset {
      role: "shader".to_owned(),
      original_ref: format!(
        "flatfekt/shaders/{}",
        name.to_string_lossy()
      ),
      packaged_path: format!(
        "{}/flatfekt/shaders/{}",
        BAKE_ASSETS_DIR,
        name.to_string_lossy()
      ),
      sha256,
      bytes: bytes.len() as u64
    });
  }
}

#[instrument(level = "info", skip_all)]
pub fn bake_scene_to_dir(
  mut cfg: flatfekt_config::RootConfig,
  scene_path: PathBuf,
  scene_bytes: Vec<u8>,
  scene_file: flatfekt_schema::SceneFile,
  req: BakeRequest
) -> anyhow::Result<BakeOutput> {
  let created_unix_secs =
    now_unix_secs()?;
  let pid = std::process::id();
  let scene_hash =
    xxhash64_hex(&scene_bytes);

  let mut output_dir =
    bake_output_dir_for_scene(
      &req.output_root,
      &scene_path,
      &scene_hash,
      created_unix_secs,
      pid
    );
  if output_dir.exists() {
    let mut suffix: u32 = 1;
    loop {
      let candidate = output_dir
        .with_file_name(format!(
          "{}-{}",
          output_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("run"),
          suffix
        ));
      if !candidate.exists() {
        tracing::warn!(
          base = %output_dir.display(),
          chosen = %candidate.display(),
          "bake output dir collision; selecting a unique run dir"
        );
        output_dir = candidate;
        break;
      }
      suffix += 1;
    }
  }
  let bake_json_path =
    output_dir.join(BAKE_JSON_FILE);
  let scene_playback_path = output_dir
    .join(BAKE_SCENE_PLAYBACK_TOML);
  let assets_dir =
    output_dir.join(BAKE_ASSETS_DIR);

  std::fs::create_dir_all(&assets_dir)
    .with_context(|| {
      format!(
        "failed to create bake assets \
         dir at {}",
        assets_dir.display()
      )
    })?;

  if scene_file
    .scene
    .sequence
    .as_ref()
    .is_some_and(|s| !s.is_empty())
  {
    return bake_aggregate_scene_to_dir(
      cfg,
      scene_path,
      scene_bytes,
      scene_file,
      req,
      output_dir,
      bake_json_path,
      scene_playback_path,
      assets_dir,
      created_unix_secs,
      pid,
      scene_hash
    );
  }

  // Force deterministic stepping for
  // baking.
  let fps = if req.fps.is_finite()
    && req.fps > 0.0
  {
    req.fps
  } else {
    60.0
  };
  let duration_secs =
    if req.duration_secs.is_finite()
      && req.duration_secs > 0.0
    {
      req.duration_secs
    } else {
      10.0
    };
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
    sim.time_scale = Some(1.0);
  }
  {
    let rt =
      cfg.runtime.get_or_insert_with(
        flatfekt_config::RuntimeConfig::default,
      );
    let tl =
      rt.timeline.get_or_insert_with(
        flatfekt_config::RuntimeTimelineConfig::default,
      );
    tl.enabled = Some(true);
    tl.deterministic = Some(true);
    tl.fixed_dt_secs = Some(1.0 / fps);
    tl.max_catchup_steps =
      tl.max_catchup_steps.or(Some(4));
  }

  // Build a playback scene snapshot:
  // - points at the bake json
  // - has no timeline/simulation so
  //   playback doesn't fight itself
  // - rewrites asset ids to packaged
  //   paths (and optionally copies
  //   assets)
  let mut scene_playback =
    scene_file.clone();
  scene_playback.scene.baked =
    Some(PathBuf::from(BAKE_JSON_FILE));
  scene_playback.scene.timeline = None;
  scene_playback.scene.simulation =
    None;

  // Ensure scene playback has a
  // duration matching the bake.
  if scene_playback
    .scene
    .playback
    .is_none()
  {
    scene_playback.scene.playback =
      Some(
        flatfekt_schema::PlaybackSpec {
          duration_secs:        None,
          loop_mode:            None,
          allow_user_input:     None,
          allow_scrub:          None,
          allow_rewind:         None,
          enable_introspection: None,
          target_fps:           None
        }
      );
  }
  if let Some(pb) = scene_playback
    .scene
    .playback
    .as_mut()
  {
    pb.duration_secs =
      Some(duration_secs);
    pb.loop_mode =
      Some("stop".to_owned());
  }

  let has_external_assets =
    !collect_asset_refs(
      &scene_playback.scene
    )
    .is_empty();

  // Package assets and rewrite ids.
  let mut assets = if req.copy_assets {
    if has_external_assets {
      let src_assets_root =
        flatfekt_assets::resolve::assets_root(
          &cfg
        )
        .context(
          "assets root must be configured for bake asset packaging",
        )?;
      let res = package_assets_and_rewrite_scene(
        &cfg,
        &src_assets_root,
        &assets_dir,
        &mut scene_playback
      )?;
      tracing::info!(scene = ?scene_playback.scene, "scene_playback updated");
      res
    } else {
      Vec::new()
    }
  } else {
    if has_external_assets {
      rewrite_asset_ids_to_paths(
        &cfg,
        &mut scene_playback
      )?;
    }
    Vec::new()
  };

  // Recursively copy the entire
  // flatfekt/shaders directory if it
  // exists. We copy it to BOTH the
  // bake root and the assets subdir to
  // ensure that hardcoded engine
  // paths ("flatfekt/shaders/...") and
  // rewritten paths ("assets/
  // flatfekt/shaders/...") both resolve
  // correctly.
  if req.copy_assets {
    package_builtin_shaders(
      &cfg,
      &output_dir,
      &assets_dir,
      &mut assets
    );
  }

  // Write scene_playback.toml
  let toml = toml::to_string_pretty(
    &scene_playback
  )
  .context(
    "failed to serialize \
     scene_playback.toml"
  )?;
  tracing::debug!(
    toml = %toml,
    "generated scene_playback.toml"
  );
  std::fs::write(
    &scene_playback_path,
    toml
  )
  .with_context(|| {
    format!(
      "failed to write {}",
      scene_playback_path.display()
    )
  })?;

  let meta = BakeMeta {
    created_unix_secs,
    tool: "flatfekt".to_owned(),
    tool_version: env!(
      "CARGO_PKG_VERSION"
    )
    .to_owned(),
    source_scene_path: scene_path
      .display()
      .to_string(),
    source_scene_xxhash64: scene_hash
      .clone()
  };
  let playback = BakePlayback {
    fps,
    dt_secs: 1.0 / fps,
    duration_secs,
    loop_mode: "stop".to_owned(),
    end_behavior: "stop".to_owned()
  };

  let assets_root =
    flatfekt_assets::resolve::assets_root(&cfg)
      .unwrap_or_else(|_| PathBuf::from("."));
  run_bake_app(
    cfg,
    scene_path.clone(),
    scene_file.clone(),
    BakeSettings {
      bake_json_path: bake_json_path
        .clone(),
      scene_playback_path:
        scene_playback_path.clone(),
      source_scene: scene_playback
        .clone(),
      playback,
      meta,
      assets,
      assets_root
    }
  )?;

  Ok(BakeOutput {
    dir: output_dir,
    bake_json_path,
    scene_playback_path
  })
}

#[allow(clippy::too_many_arguments)]
#[instrument(level = "info", skip_all)]
fn bake_aggregate_scene_to_dir(
  mut cfg: flatfekt_config::RootConfig,
  scene_path: PathBuf,
  _scene_bytes: Vec<u8>,
  scene_file: flatfekt_schema::SceneFile,
  req: BakeRequest,
  output_dir: PathBuf,
  bake_json_path: PathBuf,
  scene_playback_path: PathBuf,
  assets_dir: PathBuf,
  created_unix_secs: u64,
  pid: u32,
  scene_hash: String
) -> anyhow::Result<BakeOutput> {
  let seq = scene_file
    .scene
    .sequence
    .clone()
    .unwrap_or_default();

  let base_dir = scene_path
    .parent()
    .map(|p| p.to_path_buf())
    .unwrap_or_else(|| {
      PathBuf::from(".")
    });

  let mut clips: Vec<(PathBuf, f32)> =
    Vec::with_capacity(seq.len());
  for clip in seq {
    let path =
      if clip.path.is_absolute() {
        clip.path
      } else {
        base_dir.join(clip.path)
      };
    clips
      .push((path, clip.duration_secs));
  }

  let mut child_scenes: Vec<
    flatfekt_schema::SceneFile
  > = Vec::with_capacity(clips.len());

  let mut fps: Option<f32> = None;
  let mut res: Option<(u32, u32)> =
    None;

  for (idx, (path, expected_dur)) in
    clips.iter().enumerate()
  {
    let scene =
      flatfekt_schema::SceneFile::load_from_path(path)
        .map_err(|e| anyhow::anyhow!(e))?;

    let pb = scene
      .scene
      .playback
      .as_ref()
      .ok_or_else(|| {
        anyhow::anyhow!(
          "scene {} must have \
           [scene.playback] for \
           stitching/bake",
          path.display()
        )
      })?;

    let dur = pb
      .duration_secs
      .ok_or_else(|| {
        anyhow::anyhow!(
          "scene {} must specify \
           scene.playback.\
           duration_secs for \
           stitching/bake",
          path.display()
        )
      })?;
    if (dur - *expected_dur).abs()
      > 1e-6
    {
      anyhow::bail!(
        "scene {} duration mismatch: \
         clip.duration_secs={} but \
         scene.playback.\
         duration_secs={}",
        path.display(),
        expected_dur,
        dur
      );
    }

    let child_fps_u32 = pb
      .target_fps
      .ok_or_else(
      || {
        anyhow::anyhow!(
          "scene {} must specify \
           scene.playback.target_fps \
           for stitching/bake",
          path.display()
        )
      }
    )?;
    let child_fps =
      child_fps_u32 as f32;

    if let Some(prev) = fps {
      if (prev - child_fps).abs()
        > 0.0001
      {
        anyhow::bail!(
          "aggregate fps mismatch at \
           clip[{idx}]: expected {} \
           but got {} for {}",
          prev,
          child_fps,
          path.display()
        );
      }
    } else {
      fps = Some(child_fps);
    }

    let (w, h) = if let Some(r) =
      scene.scene.resolution
    {
      (r.width, r.height)
    } else {
      cfg
        .render_window_width()
        .zip(cfg.render_window_height())
        .ok_or_else(|| {
          anyhow::anyhow!(
            "scene {} must specify \
             scene.resolution or \
             config must set \
             render.window.width/\
             height",
            path.display()
          )
        })?
    };

    if let Some((pw, ph)) = res {
      if pw != w || ph != h {
        anyhow::bail!(
          "aggregate resolution \
           mismatch at clip[{idx}]: \
           expected {}x{} but got \
           {}x{} for {}",
          pw,
          ph,
          w,
          h,
          path.display()
        );
      }
    } else {
      res = Some((w, h));
    }

    child_scenes.push(scene);
  }

  let fps = fps.unwrap_or(60.0);
  if (req.fps - fps).abs() > 0.0001
    && req.fps.is_finite()
    && req.fps > 0.0
  {
    tracing::warn!(
      requested_fps = req.fps,
      stitched_fps = fps,
      "bake fps override does not \
       match stitched scene fps; \
       using stitched fps"
    );
  }

  // For aggregate scenes, duration
  // override is treated as a max
  // duration (trim) rather than a
  // hard requirement to match clip
  // sums.
  let requested_duration = if req
    .duration_secs
    .is_finite()
    && req.duration_secs > 0.0
  {
    req.duration_secs
  } else {
    clips.iter().map(|(_, d)| *d).sum()
  };

  let total_duration: f32 =
    clips.iter().map(|(_, d)| *d).sum();
  let duration_secs =
    requested_duration
      .min(total_duration);

  // Force deterministic stepping for
  // baking.
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
    sim.time_scale = Some(1.0);
  }
  {
    let rt =
      cfg.runtime.get_or_insert_with(
        flatfekt_config::RuntimeConfig::default,
      );
    let tl =
      rt.timeline.get_or_insert_with(
        flatfekt_config::RuntimeTimelineConfig::default,
      );
    tl.enabled = Some(true);
    tl.deterministic = Some(true);
    tl.fixed_dt_secs = Some(1.0 / fps);
    tl.max_catchup_steps =
      tl.max_catchup_steps.or(Some(4));
  }

  let mut scene_playback = child_scenes
    .first()
    .cloned()
    .unwrap_or_else(|| {
      scene_file.clone()
    });
  scene_playback.scene.baked =
    Some(PathBuf::from(BAKE_JSON_FILE));
  scene_playback.scene.timeline = None;
  scene_playback.scene.simulation =
    None;
  scene_playback.scene.sequence = None;

  if scene_playback
    .scene
    .playback
    .is_none()
  {
    scene_playback.scene.playback =
      Some(
        flatfekt_schema::PlaybackSpec {
          duration_secs:        None,
          loop_mode:            None,
          allow_user_input:     None,
          allow_scrub:          None,
          allow_rewind:         None,
          enable_introspection: None,
          target_fps:           None
        }
      );
  }
  if let Some(pb) = scene_playback
    .scene
    .playback
    .as_mut()
  {
    pb.duration_secs =
      Some(duration_secs);
    pb.loop_mode =
      Some("stop".to_owned());
    pb.target_fps = Some(fps as u32);
  }

  let has_external_assets =
    !collect_asset_refs(
      &scene_playback.scene
    )
    .is_empty();

  let assets = if req.copy_assets {
    if has_external_assets {
      let src_assets_root =
        flatfekt_assets::resolve::assets_root(
          &cfg,
        )
        .context(
          "assets root must be configured for bake asset packaging",
        )?;
      package_assets_and_rewrite_scene(
        &cfg,
        &src_assets_root,
        &assets_dir,
        &mut scene_playback
      )?
    } else {
      Vec::new()
    }
  } else {
    if has_external_assets {
      rewrite_asset_ids_to_paths(
        &cfg,
        &mut scene_playback
      )?;
    }
    Vec::new()
  };

  let toml = toml::to_string_pretty(
    &scene_playback
  )
  .context(
    "failed to serialize \
     scene_playback.toml"
  )?;
  std::fs::write(
    &scene_playback_path,
    toml
  )
  .with_context(|| {
    format!(
      "failed to write {}",
      scene_playback_path.display()
    )
  })?;

  let mut merged = BakedSimulation {
    version: BAKE_VERSION.to_owned(),
    meta: BakeMeta {
      created_unix_secs,
      tool: "flatfekt".to_owned(),
      tool_version: env!(
        "CARGO_PKG_VERSION"
      )
      .to_owned(),
      source_scene_path: scene_path
        .display()
        .to_string(),
      source_scene_xxhash64: scene_hash
    },
    playback: BakePlayback {
      fps,
      dt_secs: 1.0 / fps,
      duration_secs,
      loop_mode: "stop".to_owned(),
      end_behavior: "stop".to_owned()
    },
    assets,
    entities: HashMap::new(),
    events: Vec::new()
  };

  if req.copy_assets {
    package_builtin_shaders(
      &cfg,
      &output_dir,
      &assets_dir,
      &mut merged.assets
    );
  }

  let clips_dir =
    output_dir.join("clips");
  std::fs::create_dir_all(&clips_dir)
    .with_context(|| {
    format!(
      "failed to create aggregate \
       clip bake dir at {}",
      clips_dir.display()
    )
  })?;

  let mut t0: f32 = 0.0;
  let mut remaining = duration_secs;

  for (
    idx,
    ((clip_path, clip_dur), clip_scene)
  ) in clips
    .iter()
    .cloned()
    .zip(child_scenes.into_iter())
    .enumerate()
  {
    if remaining <= 0.0 {
      break;
    }
    let bake_this =
      clip_dur.min(remaining);

    tracing::info!(
      idx,
      path = %clip_path.display(),
      start_secs = t0,
      duration_secs = bake_this,
      "baking stitched clip"
    );

    let clip_dir = clips_dir
      .join(format!("clip-{idx:03}"));
    std::fs::create_dir_all(&clip_dir)
      .with_context(|| {
        format!(
          "failed to create clip bake \
           dir at {}",
          clip_dir.display()
        )
      })?;
    let clip_bake_path =
      clip_dir.join(BAKE_JSON_FILE);
    let clip_scene_playback_path =
      clip_dir
        .join(BAKE_SCENE_PLAYBACK_TOML);

    let playback = BakePlayback {
      fps,
      dt_secs: 1.0 / fps,
      duration_secs: bake_this,
      loop_mode: "stop".to_owned(),
      end_behavior: "stop".to_owned()
    };

    run_bake_app(
      cfg.clone(),
      clip_path.clone(),
      clip_scene.clone(),
      BakeSettings {
        bake_json_path:      clip_bake_path.clone(),
        scene_playback_path: clip_scene_playback_path,
        source_scene:        clip_scene,
        playback:            playback,
        meta:                BakeMeta {
          created_unix_secs,
          tool: "flatfekt".to_owned(),
          tool_version: env!(
            "CARGO_PKG_VERSION"
          )
          .to_owned(),
          source_scene_path: clip_path
            .display()
            .to_string(),
          source_scene_xxhash64: "0"
            .to_owned()
        },
        assets:              Vec::new(),
        assets_root:         flatfekt_assets::resolve::assets_root(&cfg)?
      }
    )?;

    let json = std::fs::read_to_string(
      &clip_bake_path
    )
    .with_context(|| {
      format!(
        "failed to read clip bake \
         json at {}",
        clip_bake_path.display()
      )
    })?;
    let mut baked_clip: BakedSimulation =
      serde_json::from_str(&json)
        .with_context(|| {
          format!(
            "failed to parse clip bake json at {}",
            clip_bake_path.display()
          )
        })?;

    merged.events.push(BakedEvent {
      t:      t0,
      action: "aggregate_clip_start"
        .to_owned(),
      target: Some(
        clip_path.display().to_string()
      )
    });

    for (id, mut ent) in
      baked_clip.entities.drain()
    {
      for kf in &mut ent.keyframes {
        kf.t += t0;
      }
      merged
        .entities
        .entry(id)
        .or_insert_with(|| {
          BakedEntity {
            keyframes: Vec::new()
          }
        })
        .keyframes
        .extend(ent.keyframes);
    }

    for mut ev in baked_clip.events {
      ev.t += t0;
      merged.events.push(ev);
    }

    t0 += bake_this;
    remaining -= bake_this;
  }

  for ent in
    merged.entities.values_mut()
  {
    ent.keyframes.sort_by(|a, b| {
      a.t.partial_cmp(&b.t).unwrap_or(
        std::cmp::Ordering::Equal
      )
    });
  }

  let json =
    serde_json::to_string_pretty(
      &merged
    )
    .context(
      "failed to serialize merged \
       bake.json"
    )?;
  std::fs::write(&bake_json_path, json)
    .with_context(|| {
      format!(
        "failed to write merged \
         bake.json at {}",
        bake_json_path.display()
      )
    })?;

  tracing::info!(
    pid,
    duration_secs,
    clips = clips.len(),
    "aggregate bake merged and saved"
  );

  Ok(BakeOutput {
    dir: output_dir,
    bake_json_path,
    scene_playback_path
  })
}

fn asset_ref_original(
  asset: &flatfekt_schema::AssetRef
) -> String {
  use flatfekt_schema::AssetRef;
  match asset {
    | AssetRef::Id {
      id
    } => {
      format!("id:{id}")
    }
    | AssetRef::Path {
      path
    } => format!("{}", path.display()),
    | AssetRef::String(s) => s.clone()
  }
}

fn collect_asset_refs<'a>(
  scene: &'a flatfekt_schema::Scene
) -> Vec<(
  &'static str,
  &'a flatfekt_schema::AssetRef
)> {
  let mut out = Vec::new();

  if let Some(effects) = &scene.effects
  {
    for eff in effects {
      if let Some(wgsl) = &eff.wgsl {
        out.push(("wgsl", wgsl));
      }
      if let Some(glsl) = &eff.glsl {
        out.push(("glsl", glsl));
      }
    }
  }

  for ent in &scene.entities {
    if let Some(sprite) = &ent.sprite {
      out
        .push(("image", &sprite.image));
    }
    if let Some(text) = &ent.text {
      if let Some(font) = &text.font {
        out.push(("font", font));
      }
      if let Some(spans) = &text.spans {
        for s in spans {
          if let Some(font) = &s.font {
            out.push(("font", font));
          }
        }
      }
      if let Some(effects) =
        &text.effects
      {
        for eff in effects {
          if let Some(shader) =
            &eff.shader
          {
            out.push(("wgsl", shader));
          }
        }
      }
    }
  }

  out
}

fn rewrite_asset_ids_to_paths(
  cfg: &flatfekt_config::RootConfig,
  scene_file: &mut flatfekt_schema::SceneFile
) -> anyhow::Result<()> {
  use flatfekt_schema::AssetRef;

  let mut replaced: usize = 0;
  let scene = &mut scene_file.scene;

  let mut rewrite = |a: &mut AssetRef| -> anyhow::Result<()> {
    let AssetRef::Id { id } = a else {
      return Ok(());
    };
    let Some(path) = cfg.asset_path_for_id(id) else {
      anyhow::bail!(
        "asset id {:?} not found in config.assets.map",
        id
      );
    };
    *a = AssetRef::Path { path: path.clone() };
    replaced += 1;
    Ok(())
  };

  if let Some(effects) =
    &mut scene.effects
  {
    for eff in effects {
      if let Some(wgsl) = &mut eff.wgsl
      {
        rewrite(wgsl)?;
      }
      if let Some(glsl) = &mut eff.glsl
      {
        rewrite(glsl)?;
      }
    }
  }
  for ent in &mut scene.entities {
    if let Some(sprite) =
      &mut ent.sprite
    {
      rewrite(&mut sprite.image)?;
    }
    if let Some(text) = &mut ent.text {
      if let Some(font) = &mut text.font
      {
        rewrite(font)?;
      }
      if let Some(spans) =
        &mut text.spans
      {
        for s in spans {
          if let Some(font) =
            &mut s.font
          {
            rewrite(font)?;
          }
        }
      }
      if let Some(effects) =
        &mut text.effects
      {
        for eff in effects {
          if let Some(shader) =
            &mut eff.shader
          {
            rewrite(shader)?;
          }
        }
      }
    }
  }

  if replaced > 0 {
    tracing::info!(
      replaced,
      "rewrote asset ids to paths"
    );
  }

  Ok(())
}

fn path_to_slash_string(
  p: &Path
) -> String {
  let mut out = String::new();
  for (idx, comp) in p
    .components()
    .filter_map(|c| match c {
      | std::path::Component::Normal(s) => {
        s.to_str().map(|s| s.to_owned())
      }
      | _ => None
    })
    .enumerate()
  {
    if idx > 0 {
      out.push('/');
    }
    out.push_str(&comp);
  }
  out
}

fn for_each_asset_ref_mut(
  scene: &mut flatfekt_schema::Scene,
  mut f: impl FnMut(
    &'static str,
    &mut flatfekt_schema::AssetRef
  ) -> anyhow::Result<()>
) -> anyhow::Result<()> {
  if let Some(effects) =
    &mut scene.effects
  {
    for eff in effects {
      if let Some(wgsl) = &mut eff.wgsl
      {
        f("wgsl", wgsl)?;
      }
      if let Some(glsl) = &mut eff.glsl
      {
        f("glsl", glsl)?;
      }
    }
  }

  for ent in &mut scene.entities {
    if let Some(sprite) =
      &mut ent.sprite
    {
      f("image", &mut sprite.image)?;
    }
    if let Some(text) = &mut ent.text {
      if let Some(font) = &mut text.font
      {
        f("font", font)?;
      }
      if let Some(spans) =
        &mut text.spans
      {
        for s in spans {
          if let Some(font) =
            &mut s.font
          {
            f("font", font)?;
          }
        }
      }
      if let Some(effects) =
        &mut text.effects
      {
        for eff in effects {
          if let Some(shader) =
            &mut eff.shader
          {
            f("wgsl", shader)?;
          }
        }
      }
    }
  }

  Ok(())
}

fn package_assets_and_rewrite_scene(
  cfg: &flatfekt_config::RootConfig,
  src_assets_root: &Path,
  dst_assets_root: &Path,
  scene_file: &mut flatfekt_schema::SceneFile
) -> anyhow::Result<Vec<BakeAsset>> {
  let mut assets: Vec<BakeAsset> =
    Vec::new();
  let mut copied: std::collections::HashSet<String> = std::collections::HashSet::new();

  rewrite_asset_ids_to_paths(
    cfg, scene_file
  )?;

  for_each_asset_ref_mut(
    &mut scene_file.scene,
    |role, asset| {
      let original =
        asset_ref_original(asset);
      tracing::info!(
        role,
        original,
        "packaging asset"
      );
      let abs = flatfekt_assets::resolve::resolve_asset_path_cfg(cfg, src_assets_root, &asset.clone()).with_context(|| {
      format!("failed to resolve asset ({role}) {original}")
    })?;

      let rel = abs
        .strip_prefix(src_assets_root)
        .unwrap_or(abs.as_path());
      let packaged_rel =
        path_to_slash_string(rel);
      let packaged_path = format!(
        "{}/{}",
        BAKE_ASSETS_DIR, packaged_rel
      );
      tracing::info!(
        packaged_path,
        "rewriting asset path"
      );

      if !copied
        .contains(&packaged_path)
      {
        let dst =
          dst_assets_root.join(rel);
        if let Some(parent) =
          dst.parent()
        {
          std::fs::create_dir_all(
            parent
          )
          .with_context(
            || {
              format!(
                "failed to create \
                 asset dir {}",
                parent.display()
              )
            }
          )?;
        }

        let bytes = std::fs::read(&abs)
          .with_context(|| {
            format!(
              "failed to read asset {}",
              abs.display()
            )
          })?;
        std::fs::write(&dst, &bytes)
          .with_context(|| {
            format!(
              "failed to write \
               packaged asset {}",
              dst.display()
            )
          })?;

        let sha256 = sha256_hex(&bytes);
        assets.push(BakeAsset {
          role: role.to_owned(),
          original_ref: original
            .clone(),
          packaged_path: packaged_path
            .clone(),
          sha256,
          bytes: bytes.len() as u64
        });
        copied.insert(
          packaged_path.clone()
        );
      }

      *asset = flatfekt_schema::AssetRef::Path {
        path: PathBuf::from(packaged_path),
      };
      Ok(())
    }
  )?;

  tracing::info!(
    packaged = assets.len(),
    "packaged scene assets"
  );
  Ok(assets)
}

#[instrument(level = "info", skip_all)]
fn run_bake_app(
  cfg: flatfekt_config::RootConfig,
  scene_path: PathBuf,
  scene_file: flatfekt_schema::SceneFile,
  settings: BakeSettings
) -> anyhow::Result<()> {
  use bevy::app::ScheduleRunnerPlugin;

  let dt =
    std::time::Duration::from_secs_f32(
      settings.playback.dt_secs
    );

  let mut app = App::new();
  app.add_plugins(MinimalPlugins.set(
    ScheduleRunnerPlugin::run_loop(dt)
  ));
  app.add_plugins(
    bevy::transform::TransformPlugin
  );

  app
    .add_message::<crate::ApplyPatch>()
    .add_message::<crate::simulation::SimTick>()
    .add_message::<bevy::app::AppExit>();

  app
    .insert_resource(ConfigRes(cfg))
    .insert_resource(ScenePathRes(scene_path))
    .insert_resource(SceneRes(scene_file))
    .insert_resource(settings.clone())
    .init_resource::<SpawnedEntities>()
    .init_resource::<EntityMap>()
    .init_resource::<crate::TimelineClock>()
    .init_resource::<crate::animation::TimelinePlan>()
    .init_resource::<crate::simulation::SimulationClock>()
    .init_resource::<crate::simulation::SimulationSeed>()
    .init_resource::<crate::simulation::SimRegionRes>()
    .init_resource::<crate::simulation::DeterminismPolicyRes>();

  // Startup
  app.add_systems(
    Startup,
    (
      init_bake_recorder,
      crate::simulation::init_simulation,
      crate::init_timeline_clock,
      crate::animation::init_timeline_plan,
      instantiate_scene_headless_for_bake,
      record_initial_frame,
    )
      .chain(),
  );

  // Update step order:
  // - advance sim + physics
  // - sync timeline time to sim
  // - dispatch timeline events (writes
  //   ApplyPatch messages)
  // - apply patches directly to ECS
  //   world (no reset)
  // - record keyframe
  // - exit when duration reached
  app.add_systems(
    Update,
    crate::simulation::simulation_driver,
  );
  app.add_systems(
    Update,
    (
      crate::simulation::gravity_system,
      crate::simulation::bounds_collision_system,
      crate::simulation::entity_collision_system,
    )
      .after(crate::simulation::simulation_driver),
  );
  app.add_systems(
    Update,
    sync_timeline_time_to_sim_time
      .after(crate::simulation::entity_collision_system),
  );
  app.add_systems(
    Update,
    crate::animation::process_timeline_events
      .after(sync_timeline_time_to_sim_time),
  );
  app.add_systems(
    Update,
    bake_apply_patch_to_world_system
      .after(crate::animation::process_timeline_events),
  );
  app.add_systems(
    Update,
    bake_recording_system.after(
      bake_apply_patch_to_world_system
    )
  );
  app.add_systems(
    Update,
    exit_and_save_on_duration
      .after(bake_recording_system)
  );

  app.run();
  Ok(())
}

#[instrument(level = "info", skip_all)]
fn init_bake_recorder(
  settings: Res<BakeSettings>,
  mut commands: Commands
) {
  let data = BakedSimulation {
    version:  BAKE_VERSION.to_owned(),
    meta:     settings.meta.clone(),
    playback: settings.playback.clone(),
    assets:   settings.assets.clone(),
    entities: HashMap::new(),
    events:   Vec::new()
  };
  commands.insert_resource(
    BakeRecorder {
      data
    }
  );
  tracing::info!(
    bake_json_path = %settings.bake_json_path.display(),
    scene_playback_path = %settings.scene_playback_path.display(),
    "initialized bake recorder"
  );
}

#[instrument(level = "trace", skip_all)]
fn sync_timeline_time_to_sim_time(
  sim: Res<
    crate::simulation::SimulationClock
  >,
  mut tl: ResMut<crate::TimelineClock>
) {
  tl.enabled = true;
  tl.playing = true;
  tl.t_secs = sim.t_secs;
  tl.dt_secs = sim.dt_secs;
}

#[instrument(level = "info", skip_all)]
pub fn instantiate_scene_headless_for_bake(
  mut commands: Commands,
  cfg: Res<ConfigRes>,
  scene: Res<SceneRes>,
  mut spawned: ResMut<SpawnedEntities>,
  mut entity_map: ResMut<EntityMap>
) {
  let scene = &scene.0.scene;

  spawned.0.clear();
  entity_map.0.clear();

  tracing::info!(
    entities = scene.entities.len(),
    "instantiating scene for bake \
     (headless)"
  );

  for ent in &scene.entities {
    let tf = crate::transform_from_spec(
      ent.transform
    );
    let mut e = commands.spawn(tf);

    // Make sure bake can record text
    // value and sprite tint.
    if let Some(text) = &ent.text {
      let v = text
        .value
        .clone()
        .unwrap_or_default();
      e.insert(Text2d::new(v));
      e.insert(crate::FlatfektText {
        font: text.font.clone()
      });
    }
    if let Some(sprite) = &ent.sprite {
      let mut s = Sprite::default();
      if let Some(tint) = sprite.tint {
        s.color =
          crate::color_from_rgba(tint);
      }
      if let Some(opacity) =
        sprite.opacity
      {
        let mut rgba =
          s.color.to_srgba();
        rgba.alpha = opacity;
        s.color = Color::Srgba(rgba);
      }
      if let (Some(w), Some(h)) =
        (sprite.width, sprite.height)
      {
        s.custom_size =
          Some(Vec2::new(w, h));
      }
      e.insert(s);
      e.insert(crate::FlatfektSprite {
        image: sprite.image.clone()
      });
    } else if let Some(shape) =
      &ent.shape
    {
      let mut s = Sprite::default();
      if let Some(c) = shape.color {
        s.color =
          crate::color_from_rgba(c);
      }
      match shape.kind.as_str() {
        | "rect" => {
          if let (Some(w), Some(h)) =
            (shape.width, shape.height)
          {
            s.custom_size =
              Some(Vec2::new(w, h));
          }
        }
        | "circle" | "polygon" => {
          if let Some(r) = shape.radius
          {
            s.custom_size = Some(
              Vec2::splat(2.0 * r)
            );
          }
        }
        | _ => {}
      }
      e.insert(s);
    } else if let Some(particles) =
      &ent.particles
    {
      e.insert(
        simulation::ParticleSystem {
          emission_rate: particles
            .emission_rate,
          lifetime:      particles
            .lifetime,
          velocity_min:  Vec2::from(
            particles.velocity_min
          ),
          velocity_max:  Vec2::from(
            particles.velocity_max
          ),
          max_particles: particles
            .max_particles,
          accumulator:   0.0
        }
      );
    } else if let Some(grid) = &ent.grid
    {
      let rule =
        match grid.rule.as_str() {
          | "conway" => {
            simulation::GridRule::Conway
          }
          | _ => {
            simulation::GridRule::Conway
          }
        };
      let cells = vec![
        0;
        (grid.width * grid.height)
          as usize
      ];
      e.insert(simulation::Grid {
        width: grid.width,
        height: grid.height,
        cell_size: grid.cell_size,
        next_cells: cells.clone(),
        cells,
        rule
      });
    }

    // Physics + collider.
    crate::insert_physics(
      &mut e,
      &cfg.0,
      ent.physics.as_ref(),
      ent.collider.as_ref()
    );

    let id = e.id();
    spawned.0.push(id);
    entity_map
      .0
      .entry(ent.id.clone())
      .or_default()
      .push(id);
  }
}

#[instrument(level = "info", skip_all)]
fn record_initial_frame(
  entity_map: Res<EntityMap>,
  materials: Option<
    Res<Assets<ColorMaterial>>
  >,
  query: Query<(
    &Transform,
    Option<&Text2d>,
    Option<&Sprite>,
    Option<
      &MeshMaterial2d<ColorMaterial>
    >,
    Option<&crate::FlatfektSprite>,
    Option<&crate::FlatfektText>
  )>,
  mut recorder: ResMut<BakeRecorder>,
  mut settings: ResMut<BakeSettings>
) {
  record_frame_at_time(
    0.0,
    &entity_map,
    materials.as_deref(),
    &query,
    &mut recorder,
    &mut settings
  );
}

#[instrument(level = "debug", skip_all)]
pub fn bake_recording_system(
  clock: Res<
    crate::simulation::SimulationClock
  >,
  entity_map: Res<EntityMap>,
  materials: Option<
    Res<Assets<ColorMaterial>>
  >,
  query: Query<(
    &Transform,
    Option<&Text2d>,
    Option<&Sprite>,
    Option<
      &MeshMaterial2d<ColorMaterial>
    >,
    Option<&crate::FlatfektSprite>,
    Option<&crate::FlatfektText>
  )>,
  mut recorder: ResMut<BakeRecorder>,
  mut settings: ResMut<BakeSettings>
) {
  if !clock.enabled || !clock.playing {
    return;
  }

  let t = clock.t_secs;
  record_frame_at_time(
    t,
    &entity_map,
    materials.as_deref(),
    &query,
    &mut recorder,
    &mut settings
  );
}

fn record_frame_at_time(
  t: f32,
  entity_map: &EntityMap,
  materials: Option<
    &Assets<ColorMaterial>
  >,
  query: &Query<(
    &Transform,
    Option<&Text2d>,
    Option<&Sprite>,
    Option<
      &MeshMaterial2d<ColorMaterial>
    >,
    Option<&crate::FlatfektSprite>,
    Option<&crate::FlatfektText>
  )>,
  recorder: &mut BakeRecorder,
  _settings: &mut BakeSettings
) {
  for (id, entities) in
    entity_map.0.iter()
  {
    let Some(entity) = entities.first()
    else {
      continue;
    };
    let Ok((
      tf,
      text,
      sprite,
      mat,
      ff_sprite,
      ff_text
    )) = query.get(*entity)
    else {
      continue;
    };

    // Record assets
    if let Some(ff_s) = ff_sprite {
      let path_str = ff_s
        .image
        .as_path()
        .map(|p| {
          p.to_string_lossy()
            .to_string()
        })
        .unwrap_or_default();
      if !path_str.is_empty()
        && !recorder
          .data
          .assets
          .iter()
          .any(|a| {
            a.original_ref == path_str
          })
      {
        recorder.data.assets.push(
          BakeAsset {
            role:          "image"
              .to_string(),
            original_ref:  path_str,
            packaged_path: String::new(
            ),
            sha256:        String::new(
            ),
            bytes:         0
          }
        );
      }
    }
    if let Some(ff_t) = ff_text {
      if let Some(font) = &ff_t.font {
        let path_str = font
          .as_path()
          .map(|p| {
            p.to_string_lossy()
              .to_string()
          })
          .unwrap_or_default();
        if !path_str.is_empty()
          && !recorder
            .data
            .assets
            .iter()
            .any(|a| {
              a.original_ref == path_str
            })
        {
          recorder.data.assets.push(
            BakeAsset {
              role:          "font"
                .to_string(),
              original_ref:  path_str,
              packaged_path:
                String::new(),
              sha256:
                String::new(),
              bytes:         0
            }
          );
        }
      }
    }

    let entry = recorder
      .data
      .entities
      .entry(id.clone())
      .or_insert_with(|| {
        BakedEntity {
          keyframes: Vec::new()
        }
      });

    let sprite_rgba =
      if let Some(s) = sprite {
        let rgba = s.color.to_srgba();
        Some([
          rgba.red, rgba.green,
          rgba.blue, rgba.alpha
        ])
      } else if let (
        Some(handle),
        Some(materials)
      ) = (mat, materials)
      {
        materials.get(handle).map(|m| {
          let rgba = m.color.to_srgba();
          [
            rgba.red, rgba.green,
            rgba.blue, rgba.alpha
          ]
        })
      } else {
        None
      };

    entry.keyframes.push(
      BakedKeyframe {
        t,
        transform: BakedTransform {
          x:     tf.translation.x,
          y:     tf.translation.y,
          z:     tf.translation.z,
          r:     tf
            .rotation
            .to_euler(EulerRot::XYZ)
            .2,
          scale: tf.scale.x
        },
        text_value: text
          .map(|t| t.0.clone()),
        sprite_rgba
      }
    );
  }
}

#[instrument(level = "info", skip_all)]
fn bake_apply_patch_to_world_system(
  mut events: MessageReader<
    crate::ApplyPatch
  >,
  cfg: Res<ConfigRes>,
  mut scene_res: ResMut<SceneRes>,
  mut entity_map: ResMut<EntityMap>,
  mut commands: Commands,
  mut q_transform: Query<
    &mut Transform
  >,
  mut q_text: Query<&mut Text2d>,
  mut q_sprite: Query<&mut Sprite>
) {
  use flatfekt_schema::ScenePatch;

  for ev in events.read() {
    tracing::info!(patch = ?ev.0, "bake: applying patch");

    // Always apply to spec.
    {
      let scene =
        &mut scene_res.0.scene;
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
            .find(|e| {
              e.id == *entity_id
            })
          {
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
          }
        }
      }
    }

    // Apply to ECS world so bake sees
    // the effect without
    // reset/reinstantiate.
    match &ev.0 {
      | ScenePatch::Add {
        entity
      } => {
        let tf =
          crate::transform_from_spec(
            entity.transform
          );
        let mut e = commands.spawn(tf);
        if let Some(text) = &entity.text
        {
          let v = text
            .value
            .clone()
            .unwrap_or_default();
          e.insert(Text2d::new(v));
        }
        if let Some(sprite) =
          &entity.sprite
        {
          let mut s = Sprite::default();
          if let Some(tint) =
            sprite.tint
          {
            s.color =
              crate::color_from_rgba(
                tint
              );
          }
          if let Some(opacity) =
            sprite.opacity
          {
            let mut rgba =
              s.color.to_srgba();
            rgba.alpha = opacity;
            s.color =
              Color::Srgba(rgba);
          }
          e.insert(s);
        }
        crate::insert_physics(
          &mut e,
          &cfg.0,
          entity.physics.as_ref(),
          entity.collider.as_ref()
        );

        let id = e.id();
        entity_map
          .0
          .entry(entity.id.clone())
          .or_default()
          .push(id);
      }
      | ScenePatch::Remove {
        entity_id
      } => {
        if let Some(list) =
          entity_map.0.remove(entity_id)
        {
          for ent in list {
            commands
              .entity(ent)
              .despawn();
          }
        }
      }
      | ScenePatch::Update {
        entity_id,
        patch
      } => {
        let Some(list) =
          entity_map.0.get(entity_id)
        else {
          continue;
        };
        for ent in list {
          if let Some(tf) =
            &patch.transform
          {
            if let Ok(mut cur) =
              q_transform.get_mut(*ent)
            {
              cur.translation.x = tf.x;
              cur.translation.y = tf.y;
              cur.translation.z =
                tf.z.unwrap_or(
                  cur.translation.z
                );
              if let Some(r) =
                tf.rotation
              {
                cur.rotation =
                  Quat::from_rotation_z(
                    r
                  );
              }
              if let Some(s) = tf.scale
              {
                cur.scale =
                  Vec3::splat(s);
              }
            }
          }
          if let Some(text) =
            &patch.text
          {
            if let Some(v) = &text.value
            {
              if let Ok(mut t) =
                q_text.get_mut(*ent)
              {
                t.0 = v.clone();
              } else {
                commands
                  .entity(*ent)
                  .insert(Text2d::new(
                    v.clone()
                  ));
              }
            }
          }
          if let Some(sprite) =
            &patch.sprite
          {
            if let Ok(mut s) =
              q_sprite.get_mut(*ent)
            {
              if let Some(tint) =
                sprite.tint
              {
                s.color = crate::color_from_rgba(tint);
              }
              if let Some(opacity) =
                sprite.opacity
              {
                let mut rgba =
                  s.color.to_srgba();
                rgba.alpha = opacity;
                s.color =
                  Color::Srgba(rgba);
              }
            }
          }
        }
      }
    }
  }
}

#[instrument(level = "info", skip_all)]
pub fn exit_and_save_on_duration(
  clock: Res<
    crate::simulation::SimulationClock
  >,
  mut recorder: ResMut<BakeRecorder>,
  settings: Res<BakeSettings>,
  mut exit: MessageWriter<
    bevy::app::AppExit
  >
) {
  if !clock.enabled {
    return;
  }
  if clock.t_secs
    < settings.playback.duration_secs
  {
    return;
  }

  match promote_bake_to_artifact(
    &mut recorder,
    &settings
  ) {
    | Ok(()) => {
      tracing::info!(
        json = %settings.bake_json_path.display(),
        playback = %settings.scene_playback_path.display(),
        "bake artifacts promoted"
      );
      exit.write(
        bevy::app::AppExit::Success
      );
    }
    | Err(err) => {
      tracing::error!(
        error = %err,
        "failed to save bake"
      );
      exit.write(
        bevy::app::AppExit::error()
      );
    }
  }
}

pub fn save_bake(
  recorder: &BakeRecorder,
  path: &PathBuf
) -> anyhow::Result<()> {
  let json =
    serde_json::to_string_pretty(
      &recorder.data
    )
    .context("serialize bake json")?;
  std::fs::write(path, json)
    .with_context(|| {
      format!(
        "write bake json {}",
        path.display()
      )
    })?;
  Ok(())
}

pub fn promote_bake_to_artifact(
  recorder: &mut BakeRecorder,
  settings: &BakeSettings
) -> anyhow::Result<()> {
  let bake_dir = settings
    .bake_json_path
    .parent()
    .unwrap_or(Path::new("."));
  let assets_dest =
    bake_dir.join(BAKE_ASSETS_DIR);

  // 1. Resolve asset metadata and copy
  //    files
  if !recorder.data.assets.is_empty() {
    std::fs::create_dir_all(
      &assets_dest
    )?;

    for asset in
      recorder.data.assets.iter_mut()
    {
      let src = settings
        .assets_root
        .join(&asset.original_ref);
      if src.exists() {
        let bytes =
          std::fs::read(&src)?;
        let hash = xxhash64_hex(&bytes);
        let ext = src
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("bin");
        let packaged_name =
          format!("{}.{}", hash, ext);

        asset.sha256 =
          Sha256::digest(&bytes)
            .iter()
            .map(|b| {
              format!("{:02x}", b)
            })
            .collect();
        asset.bytes =
          bytes.len() as u64;
        asset.packaged_path = format!(
          "{}/{}",
          BAKE_ASSETS_DIR,
          packaged_name
        );

        let dest = assets_dest
          .join(&packaged_name);
        if !dest.exists() {
          std::fs::copy(src, dest)?;
        }
      }
    }
  }

  // 2. Save the main bake JSON
  save_bake(
    recorder,
    &settings.bake_json_path
  )?;

  // 3. Generate and save the
  //    scene_playback.toml
  let mut playback_scene =
    settings.source_scene.clone();

  // Set the baked path relative to the
  // playback TOML if possible
  if let (
    Some(playback_parent),
    Some(json_parent)
  ) = (
    settings
      .scene_playback_path
      .parent(),
    settings.bake_json_path.parent()
  ) {
    if playback_parent == json_parent {
      playback_scene.scene.baked =
        settings
          .bake_json_path
          .file_name()
          .map(|n| PathBuf::from(n));
    } else {
      // Try to make it relative
      if let Ok(rel) = settings
        .bake_json_path
        .strip_prefix(playback_parent)
      {
        playback_scene.scene.baked =
          Some(rel.to_path_buf());
      } else {
        playback_scene.scene.baked =
          Some(
            settings
              .bake_json_path
              .clone()
          );
      }
    }
  } else {
    playback_scene.scene.baked = Some(
      settings.bake_json_path.clone()
    );
  }

  // Disable simulation in the playback
  // scene by removing the spec
  playback_scene.scene.simulation =
    None;

  let toml = toml::to_string_pretty(
    &playback_scene
  )
  .context(
    "serialize playback scene toml"
  )?;
  tracing::info!(toml = %toml, "promote_bake_to_artifact: generated toml");

  if let Some(parent) = settings
    .scene_playback_path
    .parent()
  {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(
    &settings.scene_playback_path,
    toml
  )
  .with_context(|| {
    format!(
      "write playback scene {}",
      settings
        .scene_playback_path
        .display()
    )
  })?;

  Ok(())
}

#[instrument(level = "info", skip_all)]
pub fn init_baked_simulation(
  cfg: Res<ConfigRes>,
  scene: Res<SceneRes>,
  scene_path: Res<ScenePathRes>,
  mut clock: ResMut<
    crate::TimelineClock
  >,
  mut commands: Commands
) {
  let Some(baked_path) =
    &scene.0.scene.baked
  else {
    return;
  };

  let baked_abs =
    if baked_path.is_absolute() {
      baked_path.clone()
    } else {
      let base = scene_path
        .0
        .parent()
        .unwrap_or(Path::new("."));
      base.join(baked_path)
    };

  tracing::info!(
    baked = %baked_abs.display(),
    "loading baked simulation"
  );

  match std::fs::read_to_string(
    &baked_abs
  ) {
    | Ok(json) => {
      match serde_json::from_str::<
        BakedSimulation
      >(&json)
      {
        | Ok(baked) => {
          if cfg
          .0
          .runtime_playback_baked_requires_timeline_clock()
        {
          clock.enabled = true;
        }
          clock.playing = true;
          clock.dt_secs =
            baked.playback.dt_secs;
          clock.duration_secs = Some(
            baked
              .playback
              .duration_secs
          );
          commands
            .insert_resource(baked);
        }
        | Err(e) => {
          tracing::error!(
            error = %e,
            "failed to parse baked simulation"
          );
        }
      }
    }
    | Err(e) => {
      tracing::error!(
        error = %e,
        "failed to read baked simulation file"
      );
    }
  }
}

#[instrument(level = "debug", skip_all)]
pub fn replay_baked_system(
  clock: Res<crate::TimelineClock>,
  agg: Option<
    Res<crate::aggregate::AggregateSceneRes>
  >,
  baked: Option<Res<BakedSimulation>>,
  entity_map: Res<EntityMap>,
  mut materials: Option<
    ResMut<Assets<ColorMaterial>>
  >,
  mut query: Query<(
    &mut Transform,
    Option<&mut Text2d>,
    Option<&mut Sprite>,
    Option<
      &MeshMaterial2d<ColorMaterial>
    >
  )>,
  mut last_applied_t: Local<
    Option<f32>
  >
) {
  let Some(baked) = baked else {
    return;
  };
  if !clock.enabled {
    return;
  }

  let t =
    crate::effective_scene_time_secs(
      clock.t_secs,
      agg.as_deref()
    );
  if let Some(prev) = *last_applied_t {
    if (prev - t).abs() < 0.000001 {
      return;
    }
  }
  *last_applied_t = Some(t);

  for (id, ent_bake) in
    baked.entities.iter()
  {
    let Some(target_entities) =
      entity_map.0.get(id)
    else {
      continue;
    };

    let idx = ent_bake
      .keyframes
      .partition_point(|k| k.t < t);

    let (k1, k2, factor) = if idx == 0 {
      let Some(first) =
        ent_bake.keyframes.first()
      else {
        continue;
      };
      (first, first, 0.0)
    } else if idx
      < ent_bake.keyframes.len()
    {
      let k1 =
        &ent_bake.keyframes[idx - 1];
      let k2 = &ent_bake.keyframes[idx];
      let denom =
        (k2.t - k1.t).max(0.000001);
      let factor = (t - k1.t) / denom;
      (k1, k2, factor.clamp(0.0, 1.0))
    } else {
      let Some(last) =
        ent_bake.keyframes.last()
      else {
        continue;
      };
      (last, last, 0.0)
    };

    for ent in target_entities {
      let Ok((
        mut tf,
        text,
        sprite,
        mat_handle
      )) = query.get_mut(*ent)
      else {
        continue;
      };

      tf.translation.x = k1.transform.x
        + (k2.transform.x
          - k1.transform.x)
          * factor;
      tf.translation.y = k1.transform.y
        + (k2.transform.y
          - k1.transform.y)
          * factor;
      tf.translation.z = k1.transform.z
        + (k2.transform.z
          - k1.transform.z)
          * factor;
      tf.rotation =
        Quat::from_rotation_z(
          k1.transform.r
            + (k2.transform.r
              - k1.transform.r)
              * factor
        );
      let s = k1.transform.scale
        + (k2.transform.scale
          - k1.transform.scale)
          * factor;
      tf.scale = Vec3::splat(s);

      if let (
        Some(v1),
        Some(v2),
        Some(mut tcomp)
      ) = (
        &k1.text_value,
        &k2.text_value,
        text
      ) {
        // If they differ, snap at
        // halfway to avoid
        // interpolating text.
        tcomp.0 = if factor < 0.5 {
          v1.clone()
        } else {
          v2.clone()
        };
      }

      let (Some(c1), Some(c2)) = (
        k1.sprite_rgba,
        k2.sprite_rgba
      ) else {
        continue;
      };
      let rgba = [
        c1[0]
          + (c2[0] - c1[0]) * factor,
        c1[1]
          + (c2[1] - c1[1]) * factor,
        c1[2]
          + (c2[2] - c1[2]) * factor,
        c1[3]
          + (c2[3] - c1[3]) * factor
      ];
      if let Some(mut scomp) = sprite {
        scomp.color = Color::srgba(
          rgba[0], rgba[1], rgba[2],
          rgba[3]
        );
      } else if let Some(handle) =
        mat_handle
      {
        if let Some(materials) =
          materials.as_deref_mut()
        {
          if let Some(mat) =
            materials.get_mut(handle)
          {
            mat.color = Color::srgba(
              rgba[0], rgba[1],
              rgba[2], rgba[3]
            );
          }
        }
      }
    }
  }
}
