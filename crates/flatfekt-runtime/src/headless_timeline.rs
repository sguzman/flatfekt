use flatfekt_config::RootConfig;
use flatfekt_schema::{
  SceneFile,
  ScenePatch,
  TimelineEvent
};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct HeadlessTimelineResult {
  pub final_time_secs: f32,
  pub steps:           u32,
  pub dispatched:      usize
}

#[derive(Debug, Clone)]
pub struct HeadlessTimelineOptions {
  pub max_steps:     u32,
  pub max_time_secs: Option<f32>
}

impl Default
  for HeadlessTimelineOptions
{
  fn default() -> Self {
    Self {
      max_steps:     60 * 10,
      max_time_secs: None
    }
  }
}

#[derive(Debug, Clone)]
struct PlannedEvent {
  time:    f32,
  action:  String,
  target:  Option<String>,
  payload: Option<toml::Value>,
  track:   Option<String>,
  label:   Option<String>,
  #[allow(dead_code)]
  group:   Option<String>,
  after:   Option<String>
}

#[instrument(level = "info", skip_all)]
pub fn run_headless_timeline(
  cfg: &RootConfig,
  scene_file: &mut SceneFile,
  opts: &HeadlessTimelineOptions
) -> anyhow::Result<
  HeadlessTimelineResult
> {
  let scene = &mut scene_file.scene;

  let timeline = match &scene.timeline {
    | Some(t) if !t.is_empty() => t,
    | _ => {
      tracing::info!(
        "scene has no timeline; \
         nothing to run"
      );
      return Ok(
        HeadlessTimelineResult {
          final_time_secs: 0.0,
          steps:           0,
          dispatched:      0
        }
      );
    }
  };

  let plan = build_plan(cfg, timeline)?;
  tracing::info!(
    planned_events = plan.len(),
    first_time =
      plan.first().map(|e| e.time),
    last_time =
      plan.last().map(|e| e.time),
    "headless timeline plan \
     initialized"
  );

  let dt = cfg
    .runtime_timeline_fixed_dt_secs();
  let mut t_secs: f32 = 0.0;
  let mut steps: u32 = 0;
  let mut cursor: usize = 0;
  let mut dispatched: usize = 0;

  let duration_cap =
    opts.max_time_secs.or(
      scene
        .playback
        .as_ref()
        .and_then(|p| p.duration_secs)
    );

  while steps < opts.max_steps {
    if let Some(cap) = duration_cap {
      if cap.is_finite()
        && cap >= 0.0
        && t_secs > cap
      {
        break;
      }
    }

    while cursor < plan.len()
      && plan[cursor].time <= t_secs
    {
      let ev = plan[cursor].clone();
      cursor += 1;
      dispatched += 1;
      dispatch_event(&ev, scene)?;
    }

    steps += 1;
    t_secs += dt;
  }

  Ok(HeadlessTimelineResult {
    final_time_secs: t_secs,
    steps,
    dispatched
  })
}

fn build_plan(
  cfg: &RootConfig,
  timeline: &[TimelineEvent]
) -> anyhow::Result<Vec<PlannedEvent>> {
  let enabled_tracks: Option<
    std::collections::HashSet<&str>
  > = cfg
    .runtime_timeline_enabled_tracks()
    .map(|tracks| {
      tracks
        .iter()
        .map(|s| s.as_str())
        .collect()
    });

  let mut events: Vec<PlannedEvent> =
    Vec::new();

  for (idx, ev) in
    timeline.iter().enumerate()
  {
    if !ev.time.is_finite()
      || ev.time < 0.0
    {
      tracing::error!(
        idx,
        time = ev.time,
        "timeline event time must be \
         finite and non-negative"
      );
      continue;
    }

    let (
      track,
      payload,
      label,
      group,
      after
    ) = parse_event_meta(ev);
    if let Some(enabled) =
      &enabled_tracks
    {
      if let Some(tr) = track.as_deref()
      {
        if !enabled.contains(tr) {
          continue;
        }
      }
    }

    events.push(PlannedEvent {
      time: ev.time,
      action: ev.action.clone(),
      target: ev.target.clone(),
      payload,
      track,
      label,
      group,
      after
    });
  }

  // Resolve "after" labels.
  let mut label_times =
    std::collections::HashMap::<
      String,
      f32
    >::new();
  for ev in &events {
    if let Some(label) = &ev.label {
      label_times
        .insert(label.clone(), ev.time);
    }
  }
  for ev in &mut events {
    let Some(after) = &ev.after else {
      continue;
    };
    let Some(base) =
      label_times.get(after).copied()
    else {
      tracing::error!(
        after = after.as_str(),
        "timeline event references \
         unknown label in \
         payload.after"
      );
      continue;
    };
    ev.time = base + ev.time;
  }

  events.sort_by(|a, b| {
    a.time
      .partial_cmp(&b.time)
      .unwrap_or(
        std::cmp::Ordering::Equal
      )
  });

  Ok(events)
}

fn parse_event_meta(
  ev: &TimelineEvent
) -> (
  Option<String>,
  Option<toml::Value>,
  Option<String>,
  Option<String>,
  Option<String>
) {
  let Some(payload) = &ev.payload
  else {
    return (
      None, None, None, None, None
    );
  };

  let track = payload
    .get("track")
    .and_then(|v| v.as_str())
    .map(|s| s.to_owned());
  let label = payload
    .get("label")
    .and_then(|v| v.as_str())
    .map(|s| s.to_owned());
  let group = payload
    .get("group")
    .and_then(|v| v.as_str())
    .map(|s| s.to_owned());
  let after = payload
    .get("after")
    .and_then(|v| v.as_str())
    .map(|s| s.to_owned());

  (
    track,
    Some(payload.clone()),
    label,
    group,
    after
  )
}

fn dispatch_event(
  ev: &PlannedEvent,
  scene: &mut flatfekt_schema::Scene
) -> anyhow::Result<()> {
  tracing::info!(
    event_time = ev.time,
    action = ev.action.as_str(),
    target = ev.target.as_deref(),
    track = ev.track.as_deref(),
    label = ev.label.as_deref(),
    "headless dispatch timeline event"
  );

  match ev.action.as_str() {
    | "apply_patch" => {
      let Some(payload) = &ev.payload
      else {
        tracing::warn!(
          "apply_patch missing payload"
        );
        return Ok(());
      };
      let patch =
        parse_scene_patch(payload)?;
      tracing::info!(
        ?patch,
        "headless apply_patch parsed"
      );
      apply_patch_to_scene(
        scene, &patch
      );
    }
    | other => {
      tracing::debug!(
        action = other,
        "headless ignoring unknown \
         timeline action"
      );
    }
  }

  Ok(())
}

fn parse_scene_patch(
  v: &toml::Value
) -> anyhow::Result<ScenePatch> {
  let s = toml::to_string(v)
    .unwrap_or_default();
  let patch: ScenePatch =
    toml::from_str(&s).map_err(
      |err| {
        anyhow::anyhow!(
          "failed to parse ScenePatch \
           payload: {err}"
        )
      }
    )?;
  Ok(patch)
}

fn apply_patch_to_scene(
  scene: &mut flatfekt_schema::Scene,
  patch: &ScenePatch
) {
  match patch {
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
      scene
        .entities
        .retain(|e| e.id != *entity_id);
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
        if let Some(tags) = &patch.tags
        {
          ent.tags = Some(tags.clone());
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
        if let Some(text) = &patch.text
        {
          ent.text = Some(text.clone());
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
          "headless patch update \
           target entity not found"
        );
      }
    }
  }
}
