use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error("failed to read scene file at {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse scene TOML at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("scene validation failed: {0}")]
    Validate(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SceneFile {
    pub scene: Scene,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub schema_version: Option<String>,
    pub entities: Vec<EntitySpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntitySpec {
    pub id: String,
    pub transform: Option<Transform2d>,
    pub sprite: Option<SpriteSpec>,
    pub text: Option<TextSpec>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Transform2d {
    pub x: f32,
    pub y: f32,
    pub rotation: Option<f32>,
    pub scale: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteSpec {
    pub image: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextSpec {
    pub value: String,
    pub font: Option<String>,
    pub size: Option<f32>,
}

impl SceneFile {
    #[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, SceneError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| SceneError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let text = String::from_utf8_lossy(&bytes);
        let file: SceneFile = toml::from_str(&text).map_err(|source| SceneError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        file.validate()?;
        Ok(file)
    }

    #[instrument(level = "info", skip_all)]
    pub fn validate(&self) -> Result<(), SceneError> {
        self.scene.validate()
    }
}

impl Scene {
    #[instrument(level = "info", skip_all)]
    pub fn validate(&self) -> Result<(), SceneError> {
        if self.entities.is_empty() {
            return Err(SceneError::Validate(
                "scene.entities must not be empty".to_owned(),
            ));
        }

        let mut ids = HashSet::<&str>::with_capacity(self.entities.len());
        for (idx, entity) in self.entities.iter().enumerate() {
            if entity.id.trim().is_empty() {
                return Err(SceneError::Validate(format!(
                    "scene.entities[{idx}].id must not be empty"
                )));
            }
            if !ids.insert(entity.id.as_str()) {
                return Err(SceneError::Validate(format!(
                    "duplicate entity id {:?} (scene.entities[{idx}])",
                    entity.id
                )));
            }

            if entity.sprite.is_none() && entity.text.is_none() {
                return Err(SceneError::Validate(format!(
                    "scene.entities[{idx}] ({:?}) must define at least one of [sprite, text]",
                    entity.id
                )));
            }

            if let Some(t) = &entity.transform {
                if !t.x.is_finite() || !t.y.is_finite() || t.z.is_some_and(|z| !z.is_finite()) {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].transform contains non-finite values"
                    )));
                }
                if t.rotation.is_some_and(|r| !r.is_finite()) {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].transform.rotation must be finite"
                    )));
                }
                if let Some(s) = t.scale {
                    if !s.is_finite() || s <= 0.0 {
                        return Err(SceneError::Validate(format!(
                            "scene.entities[{idx}].transform.scale must be > 0"
                        )));
                    }
                }
            }

            if let Some(sprite) = &entity.sprite {
                if sprite.image.trim().is_empty() {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].sprite.image must not be empty"
                    )));
                }
                if sprite.width.is_some_and(|w| !w.is_finite() || w <= 0.0) {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].sprite.width must be > 0"
                    )));
                }
                if sprite.height.is_some_and(|h| !h.is_finite() || h <= 0.0) {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].sprite.height must be > 0"
                    )));
                }
            }

            if let Some(text) = &entity.text {
                if text.value.is_empty() {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].text.value must not be empty"
                    )));
                }
                if text.size.is_some_and(|s| !s.is_finite() || s <= 0.0) {
                    return Err(SceneError::Validate(format!(
                        "scene.entities[{idx}].text.size must be > 0"
                    )));
                }
            }
        }

        Ok(())
    }
}
