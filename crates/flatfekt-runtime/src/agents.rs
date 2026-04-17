use bevy::prelude::*;
use tracing::instrument;

#[derive(Component, Debug, Clone)]
pub struct Agent {
  pub kind:  String,
  pub state: String
}

#[instrument(level = "info", skip_all)]
pub fn agent_tick_system(
  mut query: Query<(
    Entity,
    &mut Agent
  )>,
  _time: Res<Time>
) {
  for (entity, mut agent) in
    query.iter_mut()
  {
    // Simple FSM stub
    match agent.kind.as_str() {
      | "wanderer" => {
        if agent.state == "idle" {
          agent.state =
            "moving".to_string();
          tracing::info!(
            ?entity,
            "Agent started wandering"
          );
        }
      }
      | _ => {}
    }
  }
}

#[derive(Component, Debug, Clone)]
pub struct ScriptHook {
  pub on_tick: Option<String>
}

#[instrument(level = "info", skip_all)]
pub fn scripting_system(
  query: Query<(Entity, &ScriptHook)>
) {
  for (entity, hook) in query.iter() {
    if let Some(hook_name) =
      &hook.on_tick
    {
      tracing::info!(?entity, hook = %hook_name, "Executing script hook");
    }
  }
}
