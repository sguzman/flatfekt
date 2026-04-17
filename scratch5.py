import sys

content = """
#[derive(Resource, Default)]
pub struct ReloadStatus {
  pub last_error: Option<String>,
}

#[instrument(level = "info", skip_all)]
pub fn hot_reload_system(
  watcher: Option<Res<SceneFileWatcher>>,
  scene_path: Option<Res<ScenePathRes>>,
  mut scene_res: ResMut<SceneRes>,
  mut reload_status: Local<ReloadStatus>,
  mut commands: Commands,
) {
  let Some(w) = watcher else { return };
  let Some(p) = scene_path else { return };

  let mut changed = false;
  for event in w.receiver.try_iter() {
    match event.kind {
      notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
        changed = true;
      }
      _ => {}
    }
  }

  if changed {
    tracing::info!("Scene file change detected, hot-reloading...");
    match SceneFile::load_from_path(&p.0) {
      Ok(new_scene) => {
        scene_res.0 = new_scene;
        reload_status.last_error = None;
        commands.send_message(ResetScene);
        tracing::info!("Hot-reload successful");
      }
      Err(err) => {
        reload_status.last_error = Some(err.to_string());
        tracing::error!("Hot-reload failed: {}", err);
      }
    }
  }
}
"""

with open('crates/flatfekt-runtime/src/lib.rs', 'a') as f:
    f.write(content)
