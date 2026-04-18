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

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct AssetDecodePolicy {
  pub max_width:  Option<u32>,
  pub max_height: Option<u32>,
  pub max_bytes:  Option<u64>,
  pub fail_fast:  Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct SvgAssetConfig {
  pub enabled: Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct VideoAssetConfig {
  pub enabled: Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct AudioAssetConfig {
  pub enabled: Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct AssetsConfig {
  pub map: Option<
    std::collections::BTreeMap<
      String,
      PathBuf
    >
  >,
  pub hot_reload:
    Option<AssetsHotReloadConfig>,
  pub decode_policy:
    Option<AssetDecodePolicy>,
  pub svg: Option<SvgAssetConfig>,
  pub video: Option<VideoAssetConfig>,
  pub audio: Option<AudioAssetConfig>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct AssetsHotReloadConfig {
  pub enabled:           Option<bool>,
  pub debounce_ms:       Option<u64>,
  pub warn_and_continue: Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct FeaturesConfig {
  pub ui_egui:        Option<bool>,
  pub inspector_egui: Option<bool>,
  pub hot_reload:     Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RuntimeTimelineConfig {
  pub enabled:           Option<bool>,
  pub deterministic:     Option<bool>,
  pub fixed_dt_secs:     Option<f32>,
  pub max_catchup_steps: Option<u32>,
  pub enabled_tracks:
    Option<Vec<String>>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RuntimeHotReloadConfig {
  pub debounce_ms:       Option<u64>,
  pub warn_and_continue: Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RuntimePlaybackConfig {
  pub prefer_baked_over_simulation:
    Option<bool>,
  pub baked_requires_timeline_clock:
    Option<bool>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RuntimeConfig {
  pub timeline:
    Option<RuntimeTimelineConfig>,
  pub hot_reload:
    Option<RuntimeHotReloadConfig>,
  pub playback:
    Option<RuntimePlaybackConfig>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct SimulationConfig {
  pub backend:           Option<String>,
  pub enabled:           Option<bool>,
  pub deterministic:     Option<bool>,
  pub fixed_dt_secs:     Option<f32>,
  pub max_catchup_steps: Option<u32>,
  pub seed:              Option<u64>,
  pub playing:           Option<bool>,
  pub time_scale:        Option<f32>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct ExportBakeConfig {
  pub output_root: Option<PathBuf>,
  pub default_fps: Option<f32>,
  pub default_duration_secs:
    Option<f32>,
  pub copy_assets: Option<bool>,
  pub hash_algorithm: Option<String>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct ExportConfig {
  pub bake: Option<ExportBakeConfig>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct UiTimelineConfig {
  pub wheel_seek_secs: Option<f32>,
  pub wheel_seek_secs_min: Option<f32>,
  pub wheel_seek_secs_max: Option<f32>,
  pub wheel_seek_ctrl_scale:
    Option<f32>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct UiConfig {
  pub timeline:
    Option<UiTimelineConfig>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct PlatformConfig {
  pub unix_backend: Option<String>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RenderConfig {
  pub backend:    Option<String>,
  pub target_fps: Option<u32>,
  pub effects:
    Option<RenderEffectsConfig>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RenderEffectsConfig {
  pub enabled:           Option<bool>,
  pub global_effect_id:  Option<String>,
  pub render_to_texture:
    Option<RenderToTextureConfig>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RenderToTextureConfig {
  pub enabled: Option<bool>,
  pub mode:    Option<String>,
  pub scale:   Option<f32>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct AppConfig {
  pub name:       Option<String>,
  pub mode:       Option<String>,
  pub scene_path: Option<PathBuf>,
  pub assets_dir: Option<PathBuf>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct LoggingConfig {
  pub level:  Option<String>,
  pub filter: Option<String>
}

#[derive(
  Debug, Clone, Deserialize, Default,
)]
pub struct RootConfig {
  pub app:        Option<AppConfig>,
  pub logging:    Option<LoggingConfig>,
  pub platform: Option<PlatformConfig>,
  pub render:     Option<RenderConfig>,
  pub assets:     Option<AssetsConfig>,
  pub features: Option<FeaturesConfig>,
  pub runtime:    Option<RuntimeConfig>,
  pub export:     Option<ExportConfig>,
  pub ui:         Option<UiConfig>,
  pub simulation:
    Option<SimulationConfig>
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

    if let Some(assets) = &self.assets {
      if let Some(h) =
        &assets.hot_reload
      {
        if let Some(ms) = h.debounce_ms
        {
          if ms == 0 {
            return Err(
              ConfigError::Validate(
                "assets.hot_reload.\
                 debounce_ms must be \
                 >= 1"
                  .to_owned()
              )
            );
          }
        }
      }
    }

    if let Some(sim) = &self.simulation
    {
      let backend = self
        .simulation
        .as_ref()
        .and_then(|s| {
          s.backend.as_deref()
        })
        .unwrap_or("native");
      if backend != "native"
        && backend != "rapier2d"
      {
        return Err(
          ConfigError::Validate(
            format!(
              "unsupported \
               simulation.backend \
               {:?}; expected \
               \"native\" or \
               \"rapier2d\"",
              backend
            )
          )
        );
      }

      if let Some(dt) =
        sim.fixed_dt_secs
      {
        if !dt.is_finite() || dt <= 0.0
        {
          return Err(
            ConfigError::Validate(
              "simulation.\
               fixed_dt_secs must be \
               > 0"
                .to_owned()
            )
          );
        }
      }
      if let Some(steps) =
        sim.max_catchup_steps
      {
        if steps == 0 {
          return Err(
            ConfigError::Validate(
              "simulation.\
               max_catchup_steps must \
               be >= 1"
                .to_owned()
            )
          );
        }
      }
    }

    {
      let ub = self.unix_backend();
      if ub != "wayland" && ub != "x11"
      {
        return Err(
          ConfigError::Validate(
            format!(
              "unsupported \
               platform.unix_backend \
               {:?}; expected \
               \"wayland\" or \"x11\"",
              ub
            )
          )
        );
      }
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

    if let Some(ui) = &self.ui {
      if let Some(tl) = &ui.timeline {
        let secs = tl
          .wheel_seek_secs
          .unwrap_or(0.1);
        let min = tl
          .wheel_seek_secs_min
          .unwrap_or(0.005);
        let max = tl
          .wheel_seek_secs_max
          .unwrap_or(2.0);
        let scale = tl
          .wheel_seek_ctrl_scale
          .unwrap_or(1.2);

        if !secs.is_finite()
          || secs <= 0.0
        {
          return Err(
            ConfigError::Validate(
              "ui.timeline.\
               wheel_seek_secs must \
               be finite and > 0"
                .to_owned()
            )
          );
        }
        if !min.is_finite()
          || min <= 0.0
        {
          return Err(
            ConfigError::Validate(
              "ui.timeline.\
               wheel_seek_secs_min \
               must be finite and > 0"
                .to_owned()
            )
          );
        }
        if !max.is_finite() || max < min
        {
          return Err(
            ConfigError::Validate(
              "ui.timeline.\
               wheel_seek_secs_max \
               must be finite and >= \
               min"
                .to_owned()
            )
          );
        }
        if !scale.is_finite()
          || scale <= 1.0
        {
          return Err(
            ConfigError::Validate(
              "ui.timeline.\
               wheel_seek_ctrl_scale \
               must be finite and > \
               1.0"
                .to_owned()
            )
          );
        }
      }
    }

    if let Some(render) = &self.render {
      if let Some(effects) =
        &render.effects
      {
        if let Some(scale) = effects
          .render_to_texture
          .as_ref()
          .and_then(|r| r.scale)
        {
          if !scale.is_finite()
            || scale <= 0.0
          {
            return Err(
              ConfigError::Validate(
                "render.effects.\
                 render_to_texture.\
                 scale must be > 0"
                  .to_owned()
              )
            );
          }
        }
        if let Some(id) = effects
          .global_effect_id
          .as_deref()
        {
          if id.trim().is_empty() {
            return Err(
              ConfigError::Validate(
                "render.effects.\
                 global_effect_id \
                 must not be empty"
                  .to_owned()
              )
            );
          }
        }
      }
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
  pub fn render_effects_enabled(
    &self
  ) -> bool {
    self
      .render
      .as_ref()
      .and_then(|r| r.effects.as_ref())
      .and_then(|e| e.enabled)
      .unwrap_or(false)
  }

  pub fn render_effects_global_id(
    &self
  ) -> Option<&str> {
    self
      .render
      .as_ref()
      .and_then(|r| r.effects.as_ref())
      .and_then(|e| {
        e.global_effect_id.as_deref()
      })
  }

  pub fn render_effects_rtt_enabled(
    &self
  ) -> bool {
    self
      .render
      .as_ref()
      .and_then(|r| r.effects.as_ref())
      .and_then(|e| {
        e.render_to_texture.as_ref()
      })
      .and_then(|rtt| rtt.enabled)
      .unwrap_or(true)
  }

  pub fn render_effects_rtt_scale(
    &self
  ) -> f32 {
    self
      .render
      .as_ref()
      .and_then(|r| r.effects.as_ref())
      .and_then(|e| {
        e.render_to_texture.as_ref()
      })
      .and_then(|rtt| rtt.scale)
      .unwrap_or(1.0)
  }

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
  pub fn assets_hot_reload_enabled(
    &self
  ) -> bool {
    self
      .assets
      .as_ref()
      .and_then(|a| {
        a.hot_reload
          .as_ref()
          .and_then(|h| h.enabled)
      })
      .unwrap_or_else(|| {
        self
          .feature_hot_reload_enabled()
      })
  }

  pub fn assets_hot_reload_debounce_ms(
    &self
  ) -> u64 {
    self
      .assets
      .as_ref()
      .and_then(|a| {
        a.hot_reload
          .as_ref()
          .and_then(|h| h.debounce_ms)
      })
      .or_else(|| {
        self.runtime.as_ref().and_then(
          |r| {
            r.hot_reload
              .as_ref()
              .and_then(|h| {
                h.debounce_ms
              })
          }
        )
      })
      .unwrap_or(250)
  }

  pub fn assets_hot_reload_warn_and_continue(
    &self
  ) -> bool {
    self
      .assets
      .as_ref()
      .and_then(|a| {
        a.hot_reload.as_ref().and_then(
          |h| h.warn_and_continue
        )
      })
      .or_else(|| {
        self.runtime.as_ref().and_then(
          |r| {
            r.hot_reload
              .as_ref()
              .and_then(|h| {
                h.warn_and_continue
              })
          }
        )
      })
      .unwrap_or(true)
  }

  pub fn simulation_enabled(
    &self
  ) -> bool {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.enabled)
      .unwrap_or(false)
  }

  pub fn simulation_backend(
    &self
  ) -> &str {
    self
      .simulation
      .as_ref()
      .and_then(|s| {
        s.backend.as_deref()
      })
      .unwrap_or("native")
  }

  pub fn simulation_playing(
    &self
  ) -> bool {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.playing)
      .unwrap_or(true)
  }

  pub fn simulation_deterministic(
    &self
  ) -> bool {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.deterministic)
      .unwrap_or(true)
  }

  pub fn simulation_fixed_dt_secs(
    &self
  ) -> f32 {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.fixed_dt_secs)
      .unwrap_or(1.0 / 60.0)
  }

  pub fn simulation_max_catchup_steps(
    &self
  ) -> u32 {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.max_catchup_steps)
      .unwrap_or(4)
  }

  pub fn simulation_seed(&self) -> u64 {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.seed)
      .unwrap_or(0)
  }

  pub fn simulation_time_scale(
    &self
  ) -> f32 {
    self
      .simulation
      .as_ref()
      .and_then(|s| s.time_scale)
      .unwrap_or(1.0)
  }

  pub fn render_target_fps(
    &self
  ) -> u32 {
    self
      .render
      .as_ref()
      .and_then(|r| r.target_fps)
      .unwrap_or(60)
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
      .runtime_timeline_enabled_opt()
      .unwrap_or(false)
  }

  pub fn runtime_timeline_enabled_opt(
    &self
  ) -> Option<bool> {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.timeline.as_ref())
      .and_then(|t| t.enabled)
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

impl RootConfig {
  pub fn export_bake_output_root(
    &self
  ) -> PathBuf {
    self
      .export
      .as_ref()
      .and_then(|e| e.bake.as_ref())
      .and_then(|b| {
        b.output_root.clone()
      })
      .unwrap_or_else(|| {
        PathBuf::from(
          ".cache/flatfekt/scene"
        )
      })
  }

  pub fn export_bake_default_fps(
    &self
  ) -> f32 {
    self
      .export
      .as_ref()
      .and_then(|e| e.bake.as_ref())
      .and_then(|b| b.default_fps)
      .unwrap_or(60.0)
  }

  pub fn export_bake_default_duration_secs(
    &self
  ) -> f32 {
    self
      .export
      .as_ref()
      .and_then(|e| e.bake.as_ref())
      .and_then(|b| {
        b.default_duration_secs
      })
      .unwrap_or(10.0)
  }

  pub fn export_bake_copy_assets(
    &self
  ) -> bool {
    self
      .export
      .as_ref()
      .and_then(|e| e.bake.as_ref())
      .and_then(|b| b.copy_assets)
      .unwrap_or(true)
  }

  pub fn runtime_playback_prefer_baked_over_simulation(
    &self
  ) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.playback.as_ref())
      .and_then(|p| {
        p.prefer_baked_over_simulation
      })
      .unwrap_or(true)
  }

  pub fn runtime_playback_baked_requires_timeline_clock(
    &self
  ) -> bool {
    self
      .runtime
      .as_ref()
      .and_then(|r| r.playback.as_ref())
      .and_then(|p| {
        p.baked_requires_timeline_clock
      })
      .unwrap_or(true)
  }
}

impl RootConfig {
  pub fn ui_timeline_wheel_seek_secs(
    &self
  ) -> f32 {
    self
      .ui
      .as_ref()
      .and_then(|u| u.timeline.as_ref())
      .and_then(|t| t.wheel_seek_secs)
      .unwrap_or(0.1)
  }

  pub fn ui_timeline_wheel_seek_secs_min(
    &self
  ) -> f32 {
    self
      .ui
      .as_ref()
      .and_then(|u| u.timeline.as_ref())
      .and_then(|t| {
        t.wheel_seek_secs_min
      })
      .unwrap_or(0.005)
  }

  pub fn ui_timeline_wheel_seek_secs_max(
    &self
  ) -> f32 {
    self
      .ui
      .as_ref()
      .and_then(|u| u.timeline.as_ref())
      .and_then(|t| {
        t.wheel_seek_secs_max
      })
      .unwrap_or(2.0)
  }

  pub fn ui_timeline_wheel_seek_ctrl_scale(
    &self
  ) -> f32 {
    self
      .ui
      .as_ref()
      .and_then(|u| u.timeline.as_ref())
      .and_then(|t| {
        t.wheel_seek_ctrl_scale
      })
      .unwrap_or(1.2)
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
