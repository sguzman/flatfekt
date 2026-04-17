use std::path::{
  Path,
  PathBuf
};

use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
  #[error(
    "failed to read config file at \
     {path}: {source}"
  )]
  Read {
    path:   PathBuf,
    #[source]
    source: std::io::Error
  },

  #[error(
    "failed to parse TOML config at \
     {path}: {source}"
  )]
  Parse {
    path:   PathBuf,
    #[source]
    source: toml::de::Error
  },

  #[error(
    "config validation failed: {0}"
  )]
  Validate(String)
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetsConfig {
  pub map: Option<
    std::collections::BTreeMap<
      String,
      PathBuf
    >
  >
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeaturesConfig {
  pub ui_egui:        Option<bool>,
  pub inspector_egui: Option<bool>,
  pub hot_reload:     Option<bool>
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeTimelineConfig {
  pub enabled:           Option<bool>,
  pub deterministic:     Option<bool>,
  pub fixed_dt_secs:     Option<f32>,
  pub max_catchup_steps: Option<u32>,
  pub enabled_tracks:
    Option<Vec<String>>
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeHotReloadConfig {
  pub debounce_ms:       Option<u64>,
  pub warn_and_continue: Option<bool>
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
  pub timeline:
    Option<RuntimeTimelineConfig>,
  pub hot_reload:
    Option<RuntimeHotReloadConfig>
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformConfig {
  pub unix_backend: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderConfig {
  pub backend: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
  pub name:       Option<String>,
  pub mode:       Option<String>,
  pub scene_path: Option<PathBuf>,
  pub assets_dir: Option<PathBuf>
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
  pub level:  Option<String>,
  pub filter: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct RootConfig {
  pub app:      Option<AppConfig>,
  pub logging:  Option<LoggingConfig>,
  pub platform: Option<PlatformConfig>,
  pub render:   Option<RenderConfig>,
  pub assets:   Option<AssetsConfig>,
  pub features: Option<FeaturesConfig>,
  pub runtime:  Option<RuntimeConfig>
}

impl RootConfig {
  #[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
  pub fn load_from_path(
    path: impl AsRef<Path>
  ) -> Result<Self, ConfigError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)
      .map_err(|source| {
        ConfigError::Read {
          path: path.to_path_buf(),
          source
        }
      })?;
    let text =
      String::from_utf8_lossy(&bytes);
    let cfg: RootConfig =
      toml::from_str(&text).map_err(
        |source| {
          ConfigError::Parse {
            path: path.to_path_buf(),
            source
          }
        }
      )?;
    cfg.validate()?;
    Ok(cfg)
  }
}

impl RootConfig {
  pub fn unix_backend(&self) -> &str {
    self
      .platform
      .as_ref()
      .and_then(|p| {
        p.unix_backend.as_deref()
      })
      .unwrap_or("wayland")
  }

  pub fn render_backend(&self) -> &str {
    self
      .render
      .as_ref()
      .and_then(|r| {
        r.backend.as_deref()
      })
      .unwrap_or("vulkan")
  }

  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn validate(
    &self
  ) -> Result<(), ConfigError> {
    if let Some(app) = &self.app {
      if let Some(mode) = &app.mode {
        let mode = mode.as_str();
        let ok = matches!(
          mode,
          "dev" | "prod"
        );
        if !ok {
          return Err(
            ConfigError::Validate(
              format!(
                "unsupported app.mode \
                 {:?}; expected \
                 \"dev\" or \"prod\"",
                mode
              )
            )
          );
        }
      }
      if let Some(scene_path) =
        &app.scene_path
      {
        if scene_path
          .as_os_str()
          .is_empty()
        {
          return Err(
            ConfigError::Validate(
              "app.scene_path must \
               not be empty"
                .to_owned()
            )
          );
        }
      }
      if let Some(assets_dir) =
        &app.assets_dir
      {
        if assets_dir
          .as_os_str()
          .is_empty()
        {
          return Err(
            ConfigError::Validate(
              "app.assets_dir must \
               not be empty"
                .to_owned()
            )
          );
        }
      }
    }

    if let Some(assets) = &self.assets {
      if let Some(map) = &assets.map {
        for (id, path) in map {
          if id.trim().is_empty() {
            return Err(
              ConfigError::Validate(
                "assets.map keys must \
                 not be empty"
                  .to_owned()
              )
            );
          }
          validate_rel_path(
            &format!(
              "assets.map[{id:?}]"
            ),
            path
          )?;
        }
      }
    }

    if let Some(runtime) = &self.runtime
    {
      if let Some(t) = &runtime.timeline
      {
        if let Some(dt) =
          t.fixed_dt_secs
        {
          if !dt.is_finite()
            || dt <= 0.0
          {
            return Err(
              ConfigError::Validate(
                "runtime.timeline.\
                 fixed_dt_secs must \
                 be > 0"
                  .to_owned()
              )
            );
          }
        }
        if let Some(steps) =
          t.max_catchup_steps
        {
          if steps == 0 {
            return Err(
              ConfigError::Validate(
                "runtime.timeline.\
                 max_catchup_steps \
                 must be >= 1"
                  .to_owned()
              )
            );
          }
        }
        if let Some(tracks) =
          &t.enabled_tracks
        {
          for (idx, tr) in
            tracks.iter().enumerate()
          {
            if tr.trim().is_empty() {
              return Err(
                ConfigError::Validate(format!(
                  "runtime.timeline.enabled_tracks[{idx}] must not be empty"
                ))
              );
            }
          }
        }
      }

      if let Some(h) =
        &runtime.hot_reload
      {
        if let Some(ms) = h.debounce_ms
        {
          if ms == 0 {
            return Err(
              ConfigError::Validate(
                "runtime.hot_reload.\
                 debounce_ms must be \
                 >= 1"
                  .to_owned()
              )
            );
          }
        }
      }
    }

    if self.unix_backend() != "wayland"
    {
      return Err(
        ConfigError::Validate(format!(
          "unsupported \
           platform.unix_backend \
           {:?}; expected \"wayland\"",
          self.unix_backend()
        ))
      );
    }
    if self.render_backend() != "vulkan"
    {
      return Err(
        ConfigError::Validate(format!(
          "unsupported render.backend \
           {:?}; expected \"vulkan\"",
          self.render_backend()
        ))
      );
    }

    if let Some(logging) = &self.logging
    {
      if let Some(level) =
        &logging.level
      {
        let level = level.as_str();
        let ok = matches!(
          level,
          "trace"
            | "debug"
            | "info"
            | "warn"
            | "error"
        );
        if !ok {
          return Err(ConfigError::Validate(format!(
                        "unsupported logging.level {:?}; expected one of trace|debug|info|warn|error",
                        level
                    )));
        }
      }
    }

    Ok(())
  }
}

impl RootConfig {
  pub fn app_mode(&self) -> &str {
    self
      .app
      .as_ref()
      .and_then(|a| a.mode.as_deref())
      .unwrap_or("dev")
  }
}

impl RootConfig {
  pub fn asset_path_for_id(
    &self,
    id: &str
  ) -> Option<&PathBuf> {
    self
      .assets
      .as_ref()
      .and_then(|a| a.map.as_ref())
      .and_then(|m| m.get(id))
  }
}

impl RootConfig {
  pub fn feature_ui_egui_enabled(
    &self
  ) -> bool {
    self
      .features
      .as_ref()
      .and_then(|f| f.ui_egui)
      .unwrap_or(false)
  }

  pub fn feature_inspector_egui_enabled(
    &self
  ) -> bool {
    self
      .features
      .as_ref()
      .and_then(|f| f.inspector_egui)
      .unwrap_or(false)
  }

  pub fn feature_hot_reload_enabled(
    &self
  ) -> bool {
    self
      .features
      .as_ref()
      .and_then(|f| f.hot_reload)
      .unwrap_or(false)
  }

  pub fn runtime_timeline_enabled(
    &self
  ) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.enabled)
      .unwrap_or(false)
  }

  pub fn runtime_timeline_deterministic(
    &self
  ) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.deterministic)
      .unwrap_or(true)
  }

  pub fn runtime_timeline_fixed_dt_secs(
    &self
  ) -> f32 {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.fixed_dt_secs)
      .unwrap_or(1.0 / 60.0)
  }

  pub fn runtime_timeline_max_catchup_steps(
    &self
  ) -> u32 {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.max_catchup_steps)
      .unwrap_or(4)
  }

  pub fn runtime_timeline_enabled_tracks(
    &self
  ) -> Option<&[String]> {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| {
        t.enabled_tracks.as_deref()
      })
  }

  pub fn runtime_hot_reload_debounce_ms(
    &self
  ) -> u64 {
    self
      .runtime
      .as_ref()
      .and_then(|r| {
        r.hot_reload.as_ref()
      })
      .and_then(|h| h.debounce_ms)
      .unwrap_or(250)
  }

  pub fn runtime_hot_reload_warn_and_continue(
    &self
  ) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| {
        r.hot_reload.as_ref()
      })
      .and_then(|h| h.warn_and_continue)
      .unwrap_or(true)
  }
}

fn validate_rel_path(
  field: &str,
  path: &Path
) -> Result<(), ConfigError> {
  if path.as_os_str().is_empty() {
    return Err(ConfigError::Validate(
      format!(
        "{field} must not be empty"
      )
    ));
  }
  if path.is_absolute() {
    return Err(ConfigError::Validate(
      format!(
        "{field} must be relative \
         (got {path:?})"
      )
    ));
  }
  for c in path.components() {
    if matches!(
      c,
      std::path::Component::ParentDir
    ) {
      return Err(
        ConfigError::Validate(format!(
          "{field} must not contain \
           parent traversal '..' (got \
           {path:?})"
        ))
      );
    }
  }
  Ok(())
}
