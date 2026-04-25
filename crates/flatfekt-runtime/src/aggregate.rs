use std::path::PathBuf;

use bevy::prelude::*;
use tracing::instrument;

use crate::{
  ConfigRes,
  ResetScene,
  ScenePathRes,
  SceneRes,
  SeekTimeline,
  TimelineClock
};

#[derive(Debug, Clone)]
pub struct AggregateClip {
  pub path:          PathBuf,
  pub duration_secs: f32,
  pub start_secs:    f32,
  pub end_secs:      f32,
  pub fps:           u32,
  pub width:         u32,
  pub height:        u32,
  pub scene: flatfekt_schema::SceneFile
}

#[derive(Resource, Debug, Clone)]
pub struct AggregateSceneRes {
  pub clips: Vec<AggregateClip>,
  pub active_index:   usize,
  pub active_local_t: f32,
  pub total_duration: f32,
  pub fps:            u32,
  pub width:          u32,
  pub height:         u32
}

#[instrument(level = "info", skip_all)]
pub fn init_aggregate_scene(
  cfg: Res<ConfigRes>,
  scene_path: Res<ScenePathRes>,
  mut scene_res: ResMut<SceneRes>,
  mut clock: ResMut<TimelineClock>,
  mut exit: MessageWriter<
    bevy::app::AppExit
  >,
  mut commands: Commands
) {
  let Some(seq) =
    scene_res.0.scene.sequence.clone()
  else {
    return;
  };

  let base_dir = scene_path
    .0
    .parent()
    .map(|p| p.to_path_buf())
    .unwrap_or_else(|| {
      PathBuf::from(".")
    });

  let mut clips: Vec<AggregateClip> =
    Vec::with_capacity(seq.len());

  let mut fps: Option<u32> = None;
  let mut res: Option<(u32, u32)> =
    None;
  let mut t0: f32 = 0.0;

  let expected_res = scene_res
    .0
    .scene
    .resolution
    .map(|r| (r.width, r.height))
    .or_else(|| {
      cfg.0.render_window_width().zip(
        cfg.0.render_window_height()
      )
    });

  for (idx, clip) in
    seq.iter().enumerate()
  {
    let path =
      if clip.path.is_absolute() {
        clip.path.clone()
      } else {
        base_dir.join(&clip.path)
      };

    let (scene, child_fps, w, h) =
      match flatfekt_schema::SceneFile::load_from_path(&path)
        .map_err(|e| e.to_string())
        .and_then(|s| validate_child_scene(&cfg.0, &path, &s, clip.duration_secs).map(|(fps, w, h)| (s, fps, w, h)))
      {
        | Ok(v) => v,
        | Err(msg) => {
          tracing::error!(
            idx,
            path = %path.display(),
            "{msg}"
          );
          exit.write(bevy::app::AppExit::error());
          return;
        }
      };

    if let Some(prev) = fps {
      if prev != child_fps {
        tracing::error!(
          idx,
          expected = prev,
          got = child_fps,
          path = %path.display(),
          "aggregate scene fps mismatch"
        );
        exit.write(
          bevy::app::AppExit::error()
        );
        return;
      }
    } else {
      fps = Some(child_fps);
    }

    if let Some((pw, ph)) = res {
      if pw != w || ph != h {
        tracing::error!(
          idx,
          expected_w = pw,
          expected_h = ph,
          got_w = w,
          got_h = h,
          path = %path.display(),
          "aggregate scene resolution mismatch"
        );
        exit.write(
          bevy::app::AppExit::error()
        );
        return;
      }
    } else {
      res = Some((w, h));
    }

    let start = t0;
    let end = t0 + clip.duration_secs;
    clips.push(AggregateClip {
      path: path.clone(),
      duration_secs: clip.duration_secs,
      start_secs: start,
      end_secs: end,
      fps: child_fps,
      width: w,
      height: h,
      scene
    });
    t0 = end;
  }

  let fps = fps.unwrap_or(60);
  let (width, height) =
    res.unwrap_or((0, 0));
  let total_duration = t0;

  if let Some((ew, eh)) = expected_res {
    if ew != width || eh != height {
      tracing::error!(
        expected_w = ew,
        expected_h = eh,
        got_w = width,
        got_h = height,
        "aggregate scene resolution \
         does not match clip \
         resolution"
      );
      exit.write(
        bevy::app::AppExit::error()
      );
      return;
    }
  } else {
    tracing::error!(
      "aggregate scene requires a \
       resolution (scene.resolution \
       or render.window.* in config)"
    );
    exit.write(
      bevy::app::AppExit::error()
    );
    return;
  }

  clock.enabled = true;
  clock.duration_secs =
    Some(total_duration);
  clock.dt_secs = 1.0 / fps as f32;

  tracing::info!(
    clips = clips.len(),
    fps,
    width,
    height,
    total_duration,
    "initialized aggregate scene \
     sequence"
  );

  let active_index = 0usize;
  let active_local_t = 0.0;

  scene_res.0 = clips[0].scene.clone();
  commands.write_message(ResetScene);
  commands.write_message(
    SeekTimeline {
      t_secs:  0.0,
      playing: true
    }
  );

  commands.insert_resource(
    AggregateSceneRes {
      clips,
      active_index,
      active_local_t,
      total_duration,
      fps,
      width,
      height
    }
  );
}

fn validate_child_scene(
  cfg: &flatfekt_config::RootConfig,
  path: &PathBuf,
  scene: &flatfekt_schema::SceneFile,
  expected_duration: f32
) -> Result<(u32, u32, u32), String> {
  let pb = scene
    .scene
    .playback
    .as_ref()
    .ok_or_else(|| {
      format!(
        "scene {} must have \
         [scene.playback] for \
         stitching",
        path.display()
      )
    })?;
  let dur = pb
    .duration_secs
    .ok_or_else(|| {
      format!(
        "scene {} must specify \
         scene.playback.duration_secs \
         for stitching",
        path.display()
      )
    })?;
  if (dur - expected_duration).abs()
    > 1e-6
  {
    return Err(format!(
      "scene {} duration mismatch: \
       clip.duration_secs={} but \
       scene.playback.duration_secs={}",
      path.display(),
      expected_duration,
      dur
    ));
  }

  let fps =
    pb.target_fps.ok_or_else(|| {
      format!(
        "scene {} must specify \
         scene.playback.target_fps \
         for stitching",
        path.display()
      )
    })?;

  let (w, h) = if let Some(r) =
    scene.scene.resolution
  {
    (r.width, r.height)
  } else {
    cfg
      .render_window_width()
      .zip(cfg.render_window_height())
      .ok_or_else(|| {
        format!(
          "scene {} must specify \
           scene.resolution or config \
           must set render.window.\
           width/height",
          path.display()
        )
      })?
  };
  Ok((fps, w, h))
}

#[instrument(level = "trace", skip_all)]
pub fn aggregate_driver_system(
  agg: Option<
    ResMut<AggregateSceneRes>
  >,
  clock: Res<TimelineClock>,
  mut scene_res: ResMut<SceneRes>,
  mut commands: Commands
) {
  let Some(mut agg) = agg else {
    return;
  };
  let mut idx: usize = agg.active_index;
  let t = clock.t_secs;

  for (i, clip) in
    agg.clips.iter().enumerate()
  {
    if t >= clip.start_secs
      && t < clip.end_secs
    {
      idx = i;
      break;
    }
    if i == agg.clips.len() - 1
      && t >= clip.end_secs
    {
      idx = i;
      break;
    }
  }

  let local_t = (t
    - agg.clips[idx].start_secs)
    .max(0.0);

  agg.active_local_t = local_t;
  if idx != agg.active_index {
    agg.active_index = idx;
    tracing::debug!(
      idx,
      path = %agg.clips[idx].path.display(),
      "aggregate transitioning to clip"
    );
    scene_res.0 =
      agg.clips[idx].scene.clone();
    commands.write_message(ResetScene);
    commands.write_message(
      SeekTimeline {
        t_secs:  clock.t_secs,
        playing: clock.playing
      }
    );
  }
}
