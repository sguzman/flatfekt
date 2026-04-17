import sys

content = """
use flatfekt_config::ConfigRes;

#[derive(Message, Clone, Debug)]
pub struct SnapshotScene;

#[instrument(level = "info", skip_all)]
pub fn snapshot_scene_system(
  mut events: MessageReader<SnapshotScene>,
  scene_res: Res<SceneRes>,
  scene_path: Res<ScenePathRes>,
) {
  for _ in events.read() {
    let scene_name = scene_path.0.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let snapshot_dir = std::path::PathBuf::from(".cache").join("flatfekt").join("scene").join(scene_name);
    
    if let Err(e) = std::fs::create_dir_all(&snapshot_dir) {
      tracing::error!("Failed to create snapshot directory: {}", e);
      continue;
    }
    
    let snapshot_path = snapshot_dir.join("snapshot.toml");
    match toml::to_string_pretty(&scene_res.0.scene) {
      Ok(s) => {
        if let Err(e) = std::fs::write(&snapshot_path, s) {
          tracing::error!("Failed to write snapshot: {}", e);
        } else {
          tracing::info!("Scene snapshot saved to {:?}", snapshot_path);
        }
      }
      Err(e) => tracing::error!("Failed to serialize scene snapshot: {}", e),
    }
  }
}
"""

with open('crates/flatfekt-runtime/src/lib.rs', 'r') as f:
    lines = f.readlines()

lines.append(content)

with open('crates/flatfekt-runtime/src/lib.rs', 'w') as f:
    f.writelines(lines)
