use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{
  Deserialize,
  Serialize
};
use tracing::instrument;

use crate::EntityMap;
use crate::simulation::SimulationClock;

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  Resource,
  Default,
)]
pub struct BakedSimulation {
  pub version:  String,
  pub fps:      f32,
  pub duration: f32,
  pub entities:
    HashMap<String, BakedEntity>,
  pub events:   Vec<BakedEvent>
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
  pub t: f32,
  pub x: f32,
  pub y: f32,
  pub r: f32 // rotation
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct BakedEvent {
  pub t:      f32,
  pub action: String,
  pub target: Option<String>
}

#[derive(Resource, Default)]
pub struct BakeRecorder {
  pub data: BakedSimulation
}

#[instrument(level = "info", skip_all)]
pub fn bake_recording_system(
  clock: Res<SimulationClock>,
  entity_map: Res<EntityMap>,
  query: Query<&Transform>,
  mut recorder: ResMut<BakeRecorder>
) {
  if !clock.enabled || !clock.playing {
    return;
  }

  let t = clock.t_secs;
  for (id, entities) in
    entity_map.0.iter()
  {
    for entity in entities {
      if let Ok(tf) = query.get(*entity)
      {
        let entry = recorder
          .data
          .entities
          .entry(id.clone())
          .or_insert_with(|| {
            BakedEntity {
              keyframes: Vec::new()
            }
          });

        entry.keyframes.push(
          BakedKeyframe {
            t,
            x: tf.translation.x,
            y: tf.translation.y,
            r: tf
              .rotation
              .to_euler(EulerRot::XYZ)
              .2
          }
        );
      }
    }
  }
}

#[instrument(level = "trace", skip_all)]
pub fn replay_baked_system(
  clock: Res<crate::TimelineClock>,
  baked: Option<Res<BakedSimulation>>,
  entity_map: Res<EntityMap>,
  mut query: Query<&mut Transform>
) {
  let Some(baked) = baked else {
    return;
  };
  if !clock.enabled || !clock.playing {
    return;
  }

  let t = clock.t_secs;

  for (id, entities) in
    baked.entities.iter()
  {
    if let Some(target_entities) =
      entity_map.0.get(id)
    {
      // Simple linear interpolation
      // between keyframes
      let idx = entities
        .keyframes
        .partition_point(|k| k.t < t);
      if idx == 0 {
        if let Some(first) =
          entities.keyframes.first()
        {
          for ent in target_entities {
            if let Ok(mut tf) =
              query.get_mut(*ent)
            {
              tf.translation.x =
                first.x;
              tf.translation.y =
                first.y;
              tf.rotation =
                Quat::from_rotation_z(
                  first.r
                );
            }
          }
        }
      } else if idx
        < entities.keyframes.len()
      {
        let k1 =
          &entities.keyframes[idx - 1];
        let k2 =
          &entities.keyframes[idx];
        let factor =
          (t - k1.t) / (k2.t - k1.t);

        for ent in target_entities {
          if let Ok(mut tf) =
            query.get_mut(*ent)
          {
            tf.translation.x = k1.x
              + (k2.x - k1.x) * factor;
            tf.translation.y = k1.y
              + (k2.y - k1.y) * factor;
            tf.rotation =
              Quat::from_rotation_z(
                k1.r
                  + (k2.r - k1.r)
                    * factor
              );
          }
        }
      } else {
        if let Some(last) =
          entities.keyframes.last()
        {
          for ent in target_entities {
            if let Ok(mut tf) =
              query.get_mut(*ent)
            {
              tf.translation.x = last.x;
              tf.translation.y = last.y;
              tf.rotation =
                Quat::from_rotation_z(
                  last.r
                );
            }
          }
        }
      }
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
    )?;
  std::fs::write(path, json)?;
  Ok(())
}

#[instrument(level = "info", skip_all)]
pub fn init_baked_simulation(
  scene: Res<crate::SceneRes>,
  mut commands: Commands
) {
  if let Some(baked_path) =
    &scene.0.scene.baked
  {
    tracing::info!(
      path = %baked_path.display(),
      "loading baked simulation"
    );
    match std::fs::read_to_string(
      baked_path
    ) {
      | Ok(json) => {
        match serde_json::from_str::<
          BakedSimulation
        >(&json)
        {
          | Ok(baked) => {
            commands
              .insert_resource(baked);
          }
          | Err(e) => {
            tracing::error!(
              "failed to parse baked \
               simulation: {}",
              e
            );
          }
        }
      }
      | Err(e) => {
        tracing::error!(
          "failed to read baked \
           simulation file: {}",
          e
        );
      }
    }
  }
}
