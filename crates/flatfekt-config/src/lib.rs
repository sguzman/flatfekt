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
}
