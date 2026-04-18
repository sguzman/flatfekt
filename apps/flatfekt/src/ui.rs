use bevy::prelude::*;
use bevy_egui::{
  EguiContexts,
  EguiPlugin,
  EguiPrimaryContextPass,
  PrimaryEguiContext
};
use flatfekt_config::RootConfig;
use flatfekt_runtime::{
  DebugSettings,
  EntityMap,
  SceneRes,
  SeekTimeline,
  TimelineClock
};

pub fn maybe_add_ui_plugins(
  cfg: &RootConfig,
  scene_allows_inspector: bool,
  app: &mut App
) -> anyhow::Result<()> {
  if cfg.feature_ui_egui_enabled() {
    tracing::info!(
      "enabling egui control UI"
    );
    app.add_plugins(EguiPlugin::default())
      .add_systems(
        Update,
        ensure_primary_egui_context
          .run_if(no_primary_egui_context),
      )
      .add_systems(
        EguiPrimaryContextPass,
        egui_timeline_panel,
      );
  }

  if cfg
    .feature_inspector_egui_enabled()
    && scene_allows_inspector
  {
    tracing::info!(
      "enabling bevy-inspector-egui"
    );
    app.add_plugins(
      bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
    );
  }

  Ok(())
}

fn no_primary_egui_context(
  q: Query<&PrimaryEguiContext>
) -> bool {
  q.is_empty()
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

fn egui_timeline_panel(
  mut egui: EguiContexts,
  mut clock: ResMut<TimelineClock>,
  cfg: Res<flatfekt_runtime::ConfigRes>,
  time: Res<Time>,
  scene: Res<SceneRes>,
  entity_map: Res<EntityMap>,
  mut debug: ResMut<DebugSettings>,
  mut seek: MessageWriter<SeekTimeline>,
  mut wheel_seek_step: Local<
    Option<f32>
  >
) {
  let Ok(ctx) = egui.ctx_mut() else {
    return;
  };

  let wheel_seek_cfg = &cfg.0;
  let seek_min = wheel_seek_cfg
    .ui_timeline_wheel_seek_secs_min();
  let seek_max = wheel_seek_cfg
    .ui_timeline_wheel_seek_secs_max();
  let ctrl_scale = wheel_seek_cfg
    .ui_timeline_wheel_seek_ctrl_scale(
    );
  let default_step = wheel_seek_cfg
    .ui_timeline_wheel_seek_secs();
  let step = wheel_seek_step
    .get_or_insert(default_step);

  let duration =
    clock.duration_secs.unwrap_or(0.0);
  let has_duration = duration
    .is_finite()
    && duration > 0.0;

  bevy_egui::egui::TopBottomPanel::top(
    "flatfekt_top_panel"
  )
  .show(&*ctx, |ui| {
    ui.horizontal(|ui| {
      ui.label("Flatfekt");
      ui.separator();
      ui.label(format!(
        "t={:.3}s",
        clock.t_secs
      ));
      ui.separator();
      ui.label(format!(
        "FPS: {:.1}",
        1.0
          / time
            .delta_secs()
            .max(0.0001)
      ));
      if has_duration {
        ui.separator();
        ui.label(format!(
          "dur={:.3}s",
          duration
        ));
      }
    });
  });

  bevy_egui::egui::Window::new(
    "Timeline"
  )
  .resizable(true)
  .show(&*ctx, |ui| {
    ui.horizontal(|ui| {
      if ui.button("|<").clicked() {
        clock.t_secs = 0.0;
        seek.write(SeekTimeline {
          t_secs:  0.0,
          playing: clock.playing
        });
      }
      if ui.button("<<").clicked() {
        let t =
          (clock.t_secs - 1.0).max(0.0);
        clock.t_secs = t;
        seek.write(SeekTimeline {
          t_secs:  t,
          playing: clock.playing
        });
      }
      if ui.button("<").clicked() {
        clock.step_once = true;
        clock.playing = false;
        seek.write(SeekTimeline {
          t_secs:  clock.t_secs,
          playing: false
        });
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
        clock.playing = !clock.playing;
        seek.write(SeekTimeline {
          t_secs:  clock.t_secs,
          playing: clock.playing
        });
      }
      if ui.button(">").clicked() {
        clock.step_once = true;
        clock.playing = false;
        seek.write(SeekTimeline {
          t_secs:  clock.t_secs,
          playing: false
        });
      }
      if ui.button(">>").clicked() {
        let t = clock.t_secs + 1.0;
        clock.t_secs = if has_duration {
          t.min(duration)
        } else {
          t
        };
        seek.write(SeekTimeline {
          t_secs:  clock.t_secs,
          playing: clock.playing
        });
      }
      if ui.button(">|").clicked()
        && has_duration
      {
        clock.t_secs = duration;
        seek.write(SeekTimeline {
          t_secs:  duration,
          playing: clock.playing
        });
      }
    });

    ui.separator();

    if has_duration {
      let mut t = clock
        .t_secs
        .clamp(0.0, duration);
      let slider =
        bevy_egui::egui::Slider::new(
          &mut t,
          0.0..=duration
        )
        .text("time")
        .drag_value_speed(
          (duration as f64) / 250.0
        );
      let resp = ui.add(slider);
      if resp.hovered() {
        let scroll_y = ui.input(|i| {
          i.raw_scroll_delta.y
        });
        if scroll_y.abs() >= 0.01 {
          let ctrl = ui.input(|i| {
            i.modifiers.ctrl
          });
          let sign = scroll_y.signum();
          if ctrl {
            let scaled = if sign > 0.0 {
              *step * ctrl_scale
            } else {
              *step / ctrl_scale
            };
            *step = scaled.clamp(
              seek_min, seek_max
            );
            tracing::debug!(
              wheel_seek_step = *step,
              "timeline wheel seek \
               step adjusted"
            );
          } else {
            let dt = *step * sign;
            let new_t = (clock.t_secs
              + dt)
              .clamp(0.0, duration);
            clock.t_secs = new_t;
            seek.write(SeekTimeline {
              t_secs:  new_t,
              playing: clock.playing
            });
          }
        }
      }
      if resp.changed() {
        clock.t_secs = t;
        seek.write(SeekTimeline {
          t_secs:  t,
          playing: clock.playing
        });
      }
    } else {
      ui.label(
        "No duration set \
         (scene.playback.\
         duration_secs)."
      );
    }

    ui.separator();

    ui.collapsing("Debug", |ui| {
      ui.checkbox(
        &mut debug.wireframe,
        "wireframe"
      );
      ui.checkbox(
        &mut debug.draw_bounds,
        "draw bounds"
      );
      ui.label(format!(
        "entities: {}",
        entity_map.0.len()
      ));
      ui.label(format!(
        "scene has timeline: {}",
        scene
          .0
          .scene
          .timeline
          .as_ref()
          .is_some_and(|t| {
            !t.is_empty()
          })
      ));
      ui.label(format!(
        "scene has baked: {}",
        scene.0.scene.baked.is_some()
      ));
    });
  });
}
