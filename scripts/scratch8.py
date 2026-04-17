import sys

with open('crates/flatfekt-runtime/src/lib.rs', 'r') as f:
    lines = f.readlines()

# find ApplyPatch and replace to the end
idx = -1
for i, line in enumerate(lines):
    if "pub struct ApplyPatch" in line:
        idx = i - 1
        break

if idx != -1:
    lines = lines[:idx]

content = """
#[derive(Message, Clone, Debug)]
pub struct ApplyPatch(pub ScenePatch);

#[instrument(level = "info", skip_all)]
pub fn apply_patch_system(
  mut events: MessageReader<ApplyPatch>,
  mut scene_res: ResMut<SceneRes>,
  mut commands: Commands,
) {
  let mut changed = false;
  let scene = &mut scene_res.0.scene;
  
  for ev in events.read() {
    changed = true;
    match &ev.0 {
      ScenePatch::Add { entity } => {
        scene.entities.push(entity.clone());
      }
      ScenePatch::Remove { entity_id } => {
        scene.entities.retain(|e| e.id != *entity_id);
      }
      ScenePatch::Update { entity_id, patch } => {
        if let Some(ent) = scene.entities.iter_mut().find(|e| e.id == *entity_id) {
          if let Some(tags) = &patch.tags {
            ent.tags = Some(tags.clone());
          }
          if let Some(tf) = &patch.transform {
            ent.transform = Some(tf.clone());
          }
          if let Some(sprite) = &patch.sprite {
            ent.sprite = Some(sprite.clone());
          }
          if let Some(text) = &patch.text {
            ent.text = Some(text.clone());
          }
          if let Some(shape) = &patch.shape {
            ent.shape = Some(shape.clone());
          }
        }
      }
    }
  }

  if changed {
    commands.trigger(ResetScene);
  }
}
"""

lines.append(content)

with open('crates/flatfekt-runtime/src/lib.rs', 'w') as f:
    f.writelines(lines)
