use bevy::prelude::*;
use tracing::instrument;

#[derive(
  Resource, Debug, Clone, Default,
)]
pub struct ActionMap {
  pub bindings:
    std::collections::HashMap<
      String,
      Vec<KeyCode>
    >
}

#[derive(Event, Debug, Clone)]
pub struct InputAction {
  pub name: String
}

#[instrument(level = "info", skip_all)]
pub fn input_system(
  keyboard: Res<ButtonInput<KeyCode>>,
  action_map: Res<ActionMap>,
  mut commands: Commands
) {
  for (name, keys) in
    action_map.bindings.iter()
  {
    for key in keys {
      if keyboard.just_pressed(*key) {
        tracing::info!(action = %name, "Input action triggered");
        commands.trigger(InputAction {
          name: name.clone()
        });
      }
    }
  }
}

#[derive(
  Component, Debug, Clone, Default,
)]
pub struct Selectable;

#[derive(
  Component, Debug, Clone, Default,
)]
pub struct Draggable;

#[derive(Component, Debug, Clone)]
pub struct OnClick(pub String);

#[instrument(level = "info", skip_all)]
pub fn picking_system(
  mouse: Res<ButtonInput<MouseButton>>,
  windows: Query<&Window>,
  camera: Query<(
    &Camera,
    &GlobalTransform
  )>,
  query: Query<(
    Entity,
    &GlobalTransform,
    &OnClick,
    Option<&Selectable>
  )>,
  mut commands: Commands
) {
  if !mouse
    .just_pressed(MouseButton::Left)
  {
    return;
  }

  let window = windows
    .iter()
    .next()
    .expect("No window found");
  let Some((camera, camera_transform)) =
    camera.iter().next()
  else {
    return;
  };

  if let Some(cursor_pos) =
    window.cursor_position()
  {
    if let Ok(ray) = camera
      .viewport_to_world(
        camera_transform,
        cursor_pos
      )
    {
      let world_pos: Vec2 =
        ray.origin.truncate();
      for (
        entity,
        transform,
        on_click,
        _selectable
      ) in query.iter()
      {
        let pos = transform
          .translation()
          .truncate();
        // Simple radius-based picking
        // stub
        if world_pos.distance(pos)
          < 50.0
        {
          tracing::info!(?entity, action = %on_click.0, "Entity clicked");
          commands.trigger(
            InputAction {
              name: on_click.0.clone()
            }
          );
        }
      }
    }
  }
}
