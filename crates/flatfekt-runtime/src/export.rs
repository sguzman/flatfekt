use std::path::PathBuf;

use bevy::prelude::*;
use tracing::instrument;

#[derive(
  Resource, Debug, Clone, Default,
)]
pub struct ExportSettings {
  pub output_dir:         PathBuf,
  pub screenshot_pending: bool
}

#[instrument(level = "info", skip_all)]
pub fn export_system(
  mut settings: ResMut<ExportSettings>,
  _main_window: Query<&Window>
) {
  if settings.screenshot_pending {
    tracing::info!(path = %settings.output_dir.display(), "Capturing screenshot");
    // Stub: in a real Bevy app, we'd
    // use wgpu to grab the frame
    // buffer.
    settings.screenshot_pending = false;
  }
}

#[derive(
  Resource, Debug, Clone, Default,
)]
pub struct ReplayBuffer {
  pub events: Vec<(f32, String)>
}

#[instrument(level = "info", skip_all)]
pub fn replay_system(
  time: Res<Time>,
  mut buffer: ResMut<ReplayBuffer>
) {
  // Stub: record events
  if time.elapsed_secs() < 1.0 {
    buffer.events.push((
      time.elapsed_secs(),
      "tick".to_string()
    ));
  }
}
