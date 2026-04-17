import sys

content = """
#[derive(Message, Clone, Debug)]
pub struct TransitionScene(pub PathBuf);

#[instrument(level = "info", skip_all)]
pub fn scene_transition_system(
  mut events: MessageReader<TransitionScene>,
  mut scene_res: ResMut<SceneRes>,
  mut reload_status: Local<ReloadStatus>,
  mut commands: Commands,
) {
  let mut do_reset = false;
  for ev in events.read() {
    let p = &ev.0;
    tracing::info!("Transitioning to scene at {:?}", p);
    match SceneFile::load_from_path(p) {
      Ok(new_scene) => {
        scene_res.0 = new_scene;
        reload_status.last_error = None;
        do_reset = true;
        tracing::info!("Transition successful");
      }
      Err(err) => {
        reload_status.last_error = Some(err.to_string());
        tracing::error!("Scene transition failed: {}", err);
      }
    }
  }

  if do_reset {
    commands.trigger(ResetScene);
  }
}
"""

with open('crates/flatfekt-runtime/src/lib.rs', 'r') as f:
    lines = f.readlines()

lines.append(content)

with open('crates/flatfekt-runtime/src/lib.rs', 'w') as f:
    f.writelines(lines)
