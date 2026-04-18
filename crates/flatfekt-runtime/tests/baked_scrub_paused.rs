use bevy::prelude::*;
use flatfekt_runtime::bake::{
  BakeMeta,
  BakePlayback,
  BakedEntity,
  BakedKeyframe,
  BakedSimulation,
  BakedTransform,
  replay_baked_system
};
use flatfekt_runtime::{
  EntityMap,
  TimelineClock
};

#[test]
fn baked_scrub_applies_while_paused() {
  let mut app = App::new();
  app.add_plugins(MinimalPlugins);
  app.add_plugins(
    bevy::transform::TransformPlugin
  );

  app.init_resource::<TimelineClock>();
  {
    let mut clock = app
      .world_mut()
      .resource_mut::<TimelineClock>(
    );
    clock.enabled = true;
    clock.playing = false;
    clock.t_secs = 0.0;
    clock.dt_secs = 1.0 / 60.0;
  }

  let baked = BakedSimulation {
    version:  "0.2".to_owned(),
    meta:     BakeMeta {
      created_unix_secs:     0,
      tool:                  "test"
        .to_owned(),
      tool_version:          "0.0.0"
        .to_owned(),
      source_scene_path:     "test"
        .to_owned(),
      source_scene_xxhash64: "0"
        .to_owned()
    },
    playback: BakePlayback {
      fps:           60.0,
      dt_secs:       1.0 / 60.0,
      duration_secs: 1.0,
      loop_mode:     "stop".to_owned(),
      end_behavior:  "stop".to_owned()
    },
    assets:   Vec::new(),
    entities:
      std::collections::HashMap::from([
        (
          "ball".to_owned(),
          BakedEntity {
            keyframes: vec![
              BakedKeyframe {
                t:           0.0,
                transform:
                  BakedTransform {
                    x:     0.0,
                    y:     10.0,
                    z:     0.0,
                    r:     0.0,
                    scale: 1.0
                  },
                text_value:  None,
                sprite_rgba: None
              },
              BakedKeyframe {
                t:           1.0,
                transform:
                  BakedTransform {
                    x:     0.0,
                    y:     -10.0,
                    z:     0.0,
                    r:     0.0,
                    scale: 1.0
                  },
                text_value:  None,
                sprite_rgba: None
              },
            ]
          }
        )
      ]),
    events:   Vec::new()
  };
  app.insert_resource(baked);

  app.init_resource::<EntityMap>();
  let ball = app
    .world_mut()
    .spawn(Transform::default())
    .id();
  app
    .world_mut()
    .resource_mut::<EntityMap>()
    .0
    .insert("ball".to_owned(), vec![
      ball,
    ]);

  app.add_systems(
    Update,
    replay_baked_system
  );

  // At t=0, ball should be at y=10.
  app.update();
  let y0 = app
    .world()
    .entity(ball)
    .get::<Transform>()
    .unwrap()
    .translation
    .y;
  assert!(
    (y0 - 10.0).abs() < 0.001,
    "expected y0≈10, got {y0}"
  );

  // Seek while paused: updates should
  // still apply.
  {
    let mut clock = app
      .world_mut()
      .resource_mut::<TimelineClock>(
    );
    clock.t_secs = 0.5;
    clock.playing = false;
  }
  app.update();

  let y1 = app
    .world()
    .entity(ball)
    .get::<Transform>()
    .unwrap()
    .translation
    .y;
  assert!(
    y1 < y0,
    "expected scrub while paused to \
     update transform: y0={y0} y1={y1}"
  );
}
