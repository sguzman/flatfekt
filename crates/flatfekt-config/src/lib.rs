use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file at {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse TOML config at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("config validation failed: {0}")]
    Validate(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformConfig {
    pub unix_backend: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderConfig {
    pub backend: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: Option<String>,
    pub scene_path: Option<PathBuf>,
    pub assets_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RootConfig {
    pub app: Option<AppConfig>,
    pub logging: Option<LoggingConfig>,
    pub platform: Option<PlatformConfig>,
    pub render: Option<RenderConfig>,
}

impl RootConfig {
    #[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let text = String::from_utf8_lossy(&bytes);
        let cfg: RootConfig = toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        cfg.validate()?;
        Ok(cfg)
    }
}

impl RootConfig {
    pub fn unix_backend(&self) -> &str {
        self.platform
            .as_ref()
            .and_then(|p| p.unix_backend.as_deref())
            .unwrap_or("wayland")
    }

    pub fn render_backend(&self) -> &str {
        self.render
            .as_ref()
            .and_then(|r| r.backend.as_deref())
            .unwrap_or("vulkan")
    }

    #[instrument(level = "info", skip_all)]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(app) = &self.app {
            if let Some(scene_path) = &app.scene_path {
                if scene_path.as_os_str().is_empty() {
                    return Err(ConfigError::Validate(
                        "app.scene_path must not be empty".to_owned(),
                    ));
                }
            }
            if let Some(assets_dir) = &app.assets_dir {
                if assets_dir.as_os_str().is_empty() {
                    return Err(ConfigError::Validate(
                        "app.assets_dir must not be empty".to_owned(),
                    ));
                }
            }
        }

        if self.unix_backend() != "wayland" {
            return Err(ConfigError::Validate(format!(
                "unsupported platform.unix_backend {:?}; expected \"wayland\"",
                self.unix_backend()
            )));
        }
        if self.render_backend() != "vulkan" {
            return Err(ConfigError::Validate(format!(
                "unsupported render.backend {:?}; expected \"vulkan\"",
                self.render_backend()
            )));
        }

        if let Some(logging) = &self.logging {
            if let Some(level) = &logging.level {
                let level = level.as_str();
                let ok = matches!(level, "trace" | "debug" | "info" | "warn" | "error");
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
