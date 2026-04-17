use bevy::prelude::*;
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

#[instrument(level = "trace", skip_all)]
pub fn update_tweens(
  clock: Res<crate::TimelineClock>,
  mut transform_query: Query<(
    &mut Transform,
    &TransformTween
  )>,
  mut sprite_query: Query<(
    &mut Sprite,
    &ColorTween
  )>,
  mut text_query: Query<(
    &mut TextColor,
    &ColorTween
  )>,
  mut camera_query: Query<(
    &mut Transform,
    &mut Projection,
    &CameraPanZoomTween
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
    transform_query.iter_mut()
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

  for (
    mut transform,
    mut projection,
    tween
  ) in camera_query.iter_mut()
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

    if let Projection::Orthographic(
      ref mut ortho
    ) = *projection
    {
      let zoom = tween.start_zoom
        + (tween.end_zoom
          - tween.start_zoom)
          * eased;
      ortho.scale =
        1.0 / zoom.max(0.0001);
    }
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
