use bevy::prelude::*;
use flatfekt_schema::{
  ScenePatch,
  TimelineEvent
};
use tracing::instrument;

#[derive(
  Debug, Clone, Copy, PartialEq,
)]
pub enum Easing {
  Linear,
  QuadIn,
  QuadOut,
  CubicIn,
  CubicOut
}

impl Easing {
  pub fn apply(
    &self,
    t: f32
  ) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match self {
      | Easing::Linear => t,
      | Easing::QuadIn => t * t,
      | Easing::QuadOut => {
        t * (2.0 - t)
      }
      | Easing::CubicIn => t * t * t,
      | Easing::CubicOut => {
        let f = t - 1.0;
        f * f * f + 1.0
      }
    }
  }
}

#[derive(Component, Debug, Clone)]
pub struct TransformTween {
  pub start_time:      f32,
  pub duration:        f32,
  pub start_transform: Transform,
  pub end_transform:   Transform,
  pub easing:          Easing
}

#[derive(Component, Debug, Clone)]
pub struct ColorTween {
  pub start_time:  f32,
  pub duration:    f32,
  pub start_color: Color,
  pub end_color:   Color,
  pub easing:      Easing
}

#[derive(Component, Debug, Clone)]
pub struct CameraPanZoomTween {
  pub start_time:      f32,
  pub duration:        f32,
  pub start_transform: Transform,
  pub end_transform:   Transform,
  pub start_zoom:      f32,
  pub end_zoom:        f32,
  pub easing:          Easing
}

#[derive(Component, Debug, Clone)]
pub struct FadeTween {
  pub start_time:  f32,
  pub duration:    f32,
  pub start_alpha: f32,
  pub end_alpha:   f32,
  pub easing:      Easing
}

#[derive(Component, Debug, Clone)]
pub struct TypewriterEffect {
  pub start_time:    f32,
  pub chars_per_sec: f32,
  pub full_text:     String
}

#[derive(Component, Debug, Clone)]
pub struct TextLetterAnimation {
  pub effect:     LetterEffectKind,
  pub frequency:  f32,
  pub amplitude:  f32,
  pub start_time: f32
}

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum LetterEffectKind {
  Wave,
  Jitter,
  FadeIn
}

#[derive(Resource, Default)]
pub struct TimelinePlan {
  pub events: Vec<PlannedEvent>,
  pub cursor: usize
}

#[derive(Debug, Clone)]
pub struct PlannedEvent {
  pub time:    f32,
  pub action:  String,
  pub target:  Option<String>,
  pub payload: Option<toml::Value>,
  pub track:   Option<String>,
  pub label:   Option<String>,
  pub group:   Option<String>,
  pub after:   Option<String>
}

#[instrument(level = "info", skip_all)]
pub fn init_timeline_plan(
  cfg: Res<crate::ConfigRes>,
  scene: Res<crate::SceneRes>,
  mut commands: Commands
) {
  let mut plan =
    TimelinePlan::default();
  let Some(timeline) =
    &scene.0.scene.timeline
  else {
    tracing::info!(
      "scene has no timeline; no \
       events planned"
    );
    commands.insert_resource(plan);
    return;
  };

  let enabled_tracks: Option<
    std::collections::HashSet<&str>
  > = cfg
    .0
    .runtime_timeline_enabled_tracks()
    .map(|tracks| {
      tracks
        .iter()
        .map(|s| s.as_str())
        .collect()
    });

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

    plan.events.push(PlannedEvent {
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

  let mut label_times =
    std::collections::HashMap::<
      String,
      f32
    >::new();
  for ev in &plan.events {
    if let Some(label) = &ev.label {
      label_times
        .insert(label.clone(), ev.time);
    }
  }
  for ev in &mut plan.events {
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

  plan.events.sort_by(|a, b| {
    a.time
      .partial_cmp(&b.time)
      .unwrap_or(
        std::cmp::Ordering::Equal
      )
  });
  plan.cursor = 0;
  tracing::info!(
    planned_events = plan.events.len(),
    first_time = plan
      .events
      .first()
      .map(|e| e.time),
    last_time = plan
      .events
      .last()
      .map(|e| e.time),
    "timeline plan initialized"
  );
  commands.insert_resource(plan);
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

#[instrument(level = "info", skip_all)]
pub fn apply_seek_timeline(
  clock: &mut crate::TimelineClock,
  plan: &mut TimelinePlan,
  t_secs: f32,
  playing: bool
) -> Result<(), &'static str> {
  if !t_secs.is_finite() {
    return Err(
      "seek ignored: non-finite time"
    );
  }
  clock.t_secs = t_secs.max(0.0);
  clock.accumulator_secs = 0.0;
  clock.playing = playing;
  plan.cursor =
    plan.events.partition_point(|e| {
      e.time < clock.t_secs
    });
  Ok(())
}

#[instrument(level = "info", skip_all)]
pub fn seek_timeline_system(
  mut events: MessageReader<
    crate::SeekTimeline
  >,
  mut clock: ResMut<
    crate::TimelineClock
  >,
  mut plan: ResMut<TimelinePlan>
) {
  for ev in events.read() {
    match apply_seek_timeline(
      &mut clock, &mut plan, ev.t_secs,
      ev.playing
    ) {
      | Ok(()) => {
        tracing::info!(
          t_secs = clock.t_secs,
          cursor = plan.cursor,
          "timeline seek applied"
        )
      }
      | Err(msg) => {
        tracing::warn!(
          t_secs = ev.t_secs,
          "{msg}"
        )
      }
    }
  }
}

#[instrument(level = "debug", skip_all)]
pub fn process_timeline_events(
  clock: Res<crate::TimelineClock>,
  mut plan: ResMut<TimelinePlan>,
  entity_map: Res<crate::EntityMap>,
  q_transform: Query<&Transform>,
  mut last_t: Local<f32>,
  mut commands: Commands
) {
  if !clock.enabled {
    tracing::debug!(
      "timeline disabled; skipping \
       event processing"
    );
    return;
  }
  let t = clock.t_secs;
  if t < *last_t {
    tracing::info!(
      from = *last_t,
      to = t,
      "timeline time moved backwards; \
       resetting cursor"
    );
    plan.cursor = 0;
  }
  *last_t = t;

  while plan.cursor < plan.events.len()
    && plan.events[plan.cursor].time
      <= t
  {
    let ev =
      plan.events[plan.cursor].clone();
    tracing::info!(
      t_secs = t,
      cursor = plan.cursor,
      event_time = ev.time,
      action = ev.action.as_str(),
      target = ev.target.as_deref(),
      track = ev.track.as_deref(),
      label = ev.label.as_deref(),
      "dispatching timeline event"
    );
    plan.cursor += 1;
    dispatch_event(
      &ev,
      &entity_map,
      &q_transform,
      &mut commands
    );
  }
}

fn dispatch_event(
  ev: &PlannedEvent,
  entity_map: &crate::EntityMap,
  q_transform: &Query<&Transform>,
  commands: &mut Commands
) {
  match ev.action.as_str() {
    | "apply_patch" => {
      if let Some(p) = &ev.payload {
        match parse_scene_patch(p) {
          | Ok(patch) => {
            tracing::info!(
              ?patch,
              "timeline apply_patch \
               parsed"
            );
            commands.trigger(
              crate::ApplyPatch(patch)
            );
          }
          | Err(err) => {
            tracing::warn!(
              action = %ev.action,
              error = %err,
              payload = %toml::to_string_pretty(p).unwrap_or_else(|_| "<unprintable payload>".to_owned()),
              "failed to parse ScenePatch payload"
            );
          }
        }
      }
    }
    | "transition_scene" => {
      if let Some(p) = &ev.payload {
        if let Some(path) = p
          .get("path")
          .and_then(|v| v.as_str())
        {
          commands.trigger(
            crate::TransitionScene(
              std::path::PathBuf::from(
                path
              )
            )
          );
        } else {
          tracing::warn!(
            "transition_scene \
             requires payload.path"
          );
        }
      }
    }
    | "start_tween" => {
      if let Some(target) = &ev.target {
        if let Some(list) =
          entity_map.0.get(target)
        {
          for ent in list {
            if let Some(p) = &ev.payload
            {
              start_transform_tween(
                *ent,
                ev.time,
                q_transform
                  .get(*ent)
                  .ok(),
                p,
                commands
              );
            }
          }
        } else {
          tracing::warn!(
            target,
            "start_tween target not \
             found"
          );
        }
      } else {
        tracing::warn!(
          "start_tween requires target"
        );
      }
    }
    | "stop_tween" => {
      if let Some(target) = &ev.target {
        if let Some(list) =
          entity_map.0.get(target)
        {
          for ent in list {
            commands.entity(*ent).remove::<
              TransformTween
            >();
            commands
              .entity(*ent)
              .remove::<ColorTween>();
            commands
              .entity(*ent)
              .remove::<FadeTween>();
          }
        }
      }
    }
    | other => {
      tracing::debug!(
        action = other,
        "ignoring unknown timeline \
         action"
      );
    }
  }
}

fn parse_scene_patch(
  v: &toml::Value
) -> Result<ScenePatch, toml::de::Error>
{
  let s = toml::to_string(v)
    .unwrap_or_default();
  toml::from_str(&s)
}

fn start_transform_tween(
  ent: Entity,
  start_time: f32,
  start_transform: Option<&Transform>,
  payload: &toml::Value,
  commands: &mut Commands
) {
  let duration = payload
    .get("duration")
    .and_then(|v| v.as_float())
    .unwrap_or(1.0)
    as f32;
  let easing = payload
    .get("easing")
    .and_then(|v| v.as_str())
    .and_then(parse_easing)
    .unwrap_or(Easing::Linear);
  let Some(end) = payload.get("end")
  else {
    tracing::warn!(
      "start_tween requires \
       payload.end"
    );
    return;
  };
  let x = end
    .get("x")
    .and_then(|v| v.as_float())
    .unwrap_or(0.0) as f32;
  let y = end
    .get("y")
    .and_then(|v| v.as_float())
    .unwrap_or(0.0) as f32;
  let z = end
    .get("z")
    .and_then(|v| v.as_float())
    .unwrap_or(0.0) as f32;
  let start = start_transform
    .cloned()
    .unwrap_or_default();
  commands.entity(ent).insert(
    TransformTween {
      start_time,
      duration: duration.max(0.0001),
      start_transform: start,
      end_transform:
        Transform::from_translation(
          Vec3::new(x, y, z)
        ),
      easing
    }
  );
}

fn parse_easing(
  s: &str
) -> Option<Easing> {
  Some(match s {
    | "linear" => Easing::Linear,
    | "quad_in" => Easing::QuadIn,
    | "quad_out" => Easing::QuadOut,
    | "cubic_in" => Easing::CubicIn,
    | "cubic_out" => Easing::CubicOut,
    | _ => return None
  })
}

#[instrument(level = "trace", skip_all)]
pub fn update_tweens(
  clock: Res<crate::TimelineClock>,
  mut transform_and_camera: ParamSet<(
    Query<(
      &mut Transform,
      &TransformTween
    )>,
    Query<(
      &mut Transform,
      &mut Projection,
      &CameraPanZoomTween
    )>
  )>,
  mut sprite_query: Query<(
    &mut Sprite,
    &ColorTween
  )>,
  mut text_query: Query<(
    &mut TextColor,
    &ColorTween
  )>,
  mut fade_sprite_query: Query<
    (&mut Sprite, &FadeTween),
    Without<ColorTween>
  >,
  mut fade_text_query: Query<
    (&mut TextColor, &FadeTween),
    Without<ColorTween>
  >
) {
  if !clock.enabled || !clock.playing {
    return;
  }
  let t_secs = clock.t_secs;

  for (mut transform, tween) in
    transform_and_camera.p0().iter_mut()
  {
    let mut progress =
      if tween.duration > 0.0 {
        (t_secs - tween.start_time)
          / tween.duration
      } else {
        1.0
      };
    progress = progress.clamp(0.0, 1.0);
    let eased =
      tween.easing.apply(progress);

    transform.translation = tween
      .start_transform
      .translation
      .lerp(
        tween.end_transform.translation,
        eased
      );
    transform.rotation = tween
      .start_transform
      .rotation
      .slerp(
        tween.end_transform.rotation,
        eased
      );
    transform.scale =
      tween.start_transform.scale.lerp(
        tween.end_transform.scale,
        eased
      );
  }

  for (
    mut transform,
    mut proj,
    tween
  ) in
    transform_and_camera.p1().iter_mut()
  {
    let progress = ((t_secs
      - tween.start_time)
      / tween.duration.max(0.0001))
    .clamp(0.0, 1.0);
    let eased =
      tween.easing.apply(progress);

    transform.translation = tween
      .start_transform
      .translation
      .lerp(
        tween.end_transform.translation,
        eased
      );
    transform.scale =
      tween.start_transform.scale.lerp(
        tween.end_transform.scale,
        eased
      );

    let start = tween.start_zoom;
    let end = tween.end_zoom;
    let zoom = start.lerp(end, eased);
    let mut ortho =
      OrthographicProjection::default_2d();
    ortho.scale =
      1.0 / zoom.max(0.0001);
    *proj =
      Projection::Orthographic(ortho);
  }

  for (mut sprite, tween) in
    sprite_query.iter_mut()
  {
    let progress = ((t_secs
      - tween.start_time)
      / tween.duration.max(0.0001))
    .clamp(0.0, 1.0);
    let eased =
      tween.easing.apply(progress);
    let srgba_start =
      tween.start_color.to_srgba();
    let srgba_end =
      tween.end_color.to_srgba();
    sprite.color = Color::srgba(
      srgba_start
        .red
        .lerp(srgba_end.red, eased),
      srgba_start
        .green
        .lerp(srgba_end.green, eased),
      srgba_start
        .blue
        .lerp(srgba_end.blue, eased),
      srgba_start
        .alpha
        .lerp(srgba_end.alpha, eased)
    );
  }

  for (mut text_color, tween) in
    text_query.iter_mut()
  {
    let progress = ((t_secs
      - tween.start_time)
      / tween.duration.max(0.0001))
    .clamp(0.0, 1.0);
    let eased =
      tween.easing.apply(progress);
    let srgba_start =
      tween.start_color.to_srgba();
    let srgba_end =
      tween.end_color.to_srgba();
    text_color.0 = Color::srgba(
      srgba_start
        .red
        .lerp(srgba_end.red, eased),
      srgba_start
        .green
        .lerp(srgba_end.green, eased),
      srgba_start
        .blue
        .lerp(srgba_end.blue, eased),
      srgba_start
        .alpha
        .lerp(srgba_end.alpha, eased)
    );
  }

  for (mut sprite, tween) in
    fade_sprite_query.iter_mut()
  {
    let progress = ((t_secs
      - tween.start_time)
      / tween.duration.max(0.0001))
    .clamp(0.0, 1.0);
    let eased =
      tween.easing.apply(progress);
    let mut srgba =
      sprite.color.to_srgba();
    srgba.alpha = tween.start_alpha
      + (tween.end_alpha
        - tween.start_alpha)
        * eased;
    sprite.color = Color::Srgba(srgba);
  }
  for (mut text_color, tween) in
    fade_text_query.iter_mut()
  {
    let progress = ((t_secs
      - tween.start_time)
      / tween.duration.max(0.0001))
    .clamp(0.0, 1.0);
    let eased =
      tween.easing.apply(progress);
    let mut srgba =
      text_color.0.to_srgba();
    srgba.alpha = tween.start_alpha
      + (tween.end_alpha
        - tween.start_alpha)
        * eased;
    text_color.0 = Color::Srgba(srgba);
  }
}

#[instrument(level = "trace", skip_all)]
pub fn update_typewriter(
  clock: Res<crate::TimelineClock>,
  mut query: Query<(
    &TypewriterEffect,
    &mut Text2d
  )>
) {
  if !clock.enabled || !clock.playing {
    return;
  }
  let t_secs = clock.t_secs;
  for (effect, mut text) in
    query.iter_mut()
  {
    let elapsed =
      t_secs - effect.start_time;
    if elapsed < 0.0 {
      if !text.0.is_empty() {
        text.0 = String::new();
      }
      continue;
    }
    let char_count = (elapsed
      * effect.chars_per_sec)
      as usize;
    let new_val = effect
      .full_text
      .chars()
      .take(char_count)
      .collect::<String>();
    if text.0 != new_val {
      text.0 = new_val;
    }
  }
}
