#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::{
  Path,
  PathBuf
};

use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AssetRef {
  Path { path: PathBuf },
  Id { id: String },
  String(String)
}

impl AssetRef {
  pub fn as_path(
    &self
  ) -> Option<&Path> {
    match self {
      | AssetRef::Path {
        path
      } => Some(path.as_path()),
      | AssetRef::String(s) => {
        Some(Path::new(s))
      }
      | AssetRef::Id {
        ..
      } => None
    }
  }
}

#[derive(
  Debug, Clone, Copy, Deserialize,
)]
pub struct ColorRgba {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: Option<f32>
}

#[derive(Debug, thiserror::Error)]
pub enum SceneError {
  #[error(
    "failed to read scene file at \
     {path}: {source}"
  )]
  Read {
    path:   PathBuf,
    #[source]
    source: std::io::Error
  },

  #[error(
    "failed to parse scene TOML at \
     {path}: {source}"
  )]
  Parse {
    path:   PathBuf,
    #[source]
    source: toml::de::Error
  },

  #[error(
    "scene validation failed: {0}"
  )]
  Validate(String)
}

#[derive(Debug, Clone, Deserialize)]
pub struct SceneFile {
  pub scene: Scene
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
  pub schema_version: Option<String>,
  pub camera: Option<CameraSpec>,
  pub background:
    Option<BackgroundSpec>,
  pub entities:       Vec<EntitySpec>
}

#[derive(Debug, Clone, Deserialize)]
pub struct CameraSpec {
  pub x:           Option<f32>,
  pub y:           Option<f32>,
  pub zoom:        Option<f32>,
  pub clear_color: Option<ColorRgba>
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackgroundSpec {
  pub clear_color: Option<ColorRgba>
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntitySpec {
  pub id:        String,
  pub transform: Option<Transform2d>,
  pub sprite:    Option<SpriteSpec>,
  pub text:      Option<TextSpec>
}

#[derive(
  Debug, Clone, Copy, Deserialize,
)]
pub struct Transform2d {
  pub x:        f32,
  pub y:        f32,
  pub rotation: Option<f32>,
  pub scale:    Option<f32>,
  pub z:        Option<f32>
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteSpec {
  pub image:  AssetRef,
  pub width:  Option<f32>,
  pub height: Option<f32>,
  pub anchor: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextSpec {
  pub value:  String,
  pub font:   Option<AssetRef>,
  pub size:   Option<f32>,
  pub color:  Option<ColorRgba>,
  pub anchor: Option<String>,
  pub align:  Option<String>
}

impl SceneFile {
  #[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
  pub fn load_from_path(
    path: impl AsRef<Path>
  ) -> Result<Self, SceneError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)
      .map_err(|source| {
        SceneError::Read {
          path: path.to_path_buf(),
          source
        }
      })?;
    let text =
      String::from_utf8_lossy(&bytes);
    let file: SceneFile =
      toml::from_str(&text).map_err(
        |source| {
          SceneError::Parse {
            path: path.to_path_buf(),
            source
          }
        }
      )?;
    file.validate()?;
    Ok(file)
  }

  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn validate(
    &self
  ) -> Result<(), SceneError> {
    self.scene.validate()
  }
}

impl Scene {
  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn validate(
    &self
  ) -> Result<(), SceneError> {
    if self.entities.is_empty() {
      return Err(SceneError::Validate(
        "scene.entities must not be \
         empty"
          .to_owned()
      ));
    }

    if let Some(camera) = &self.camera {
      if camera
        .x
        .is_some_and(|v| !v.is_finite())
        || camera.y.is_some_and(|v| {
          !v.is_finite()
        })
      {
        return Err(
          SceneError::Validate(
            "scene.camera x/y must be \
             finite"
              .to_owned()
          )
        );
      }
      if let Some(zoom) = camera.zoom {
        if !zoom.is_finite()
          || zoom <= 0.0
        {
          return Err(
            SceneError::Validate(
              "scene.camera.zoom must \
               be > 0"
                .to_owned()
            )
          );
        }
      }
      if let Some(c) =
        camera.clear_color
      {
        validate_color(
          "scene.camera.clear_color",
          &c
        )?;
      }
    }
    if let Some(bg) = &self.background {
      if let Some(c) = bg.clear_color {
        validate_color(
          "scene.background.\
           clear_color",
          &c
        )?;
      }
    }

    let mut ids =
      HashSet::<&str>::with_capacity(
        self.entities.len()
      );
    for (idx, entity) in
      self.entities.iter().enumerate()
    {
      if entity.id.trim().is_empty() {
        return Err(
          SceneError::Validate(
            format!(
              "scene.entities[{idx}].\
               id must not be empty"
            )
          )
        );
      }
      if !ids.insert(entity.id.as_str())
      {
        return Err(
          SceneError::Validate(
            format!(
              "duplicate entity id \
               {:?} (scene.\
               entities[{idx}])",
              entity.id
            )
          )
        );
      }

      if entity.sprite.is_none()
        && entity.text.is_none()
      {
        return Err(
          SceneError::Validate(
            format!(
              "scene.entities[{idx}] \
               ({:?}) must define at \
               least one of [sprite, \
               text]",
              entity.id
            )
          )
        );
      }

      if let Some(t) = &entity.transform
      {
        if !t.x.is_finite()
          || !t.y.is_finite()
          || t.z.is_some_and(|z| {
            !z.is_finite()
          })
        {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].transform contains non-finite values"
          )));
        }
        if t.rotation.is_some_and(|r| {
          !r.is_finite()
        }) {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].transform.rotation must be finite"
          )));
        }
        if let Some(s) = t.scale {
          if !s.is_finite() || s <= 0.0
          {
            return Err(SceneError::Validate(format!(
              "scene.entities[{idx}].transform.scale must be > 0"
            )));
          }
        }
      }

      if let Some(sprite) =
        &entity.sprite
      {
        let path = sprite.image.as_path().ok_or_else(|| {
          SceneError::Validate(format!(
            "scene.entities[{idx}].sprite.image must be a path (id indirection not implemented yet)"
          ))
        })?;
        if path.as_os_str().is_empty() {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].sprite.image must not be empty"
          )));
        }
        if sprite.width.is_some_and(
          |w| {
            !w.is_finite() || w <= 0.0
          }
        ) {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].sprite.width must be > 0"
          )));
        }
        if sprite.height.is_some_and(
          |h| {
            !h.is_finite() || h <= 0.0
          }
        ) {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].sprite.height must be > 0"
          )));
        }
        if let Some(anchor) =
          sprite.anchor.as_deref()
        {
          validate_anchor(
            &format!(
              "scene.entities[{idx}].\
               sprite.anchor"
            ),
            anchor
          )?;
        }
      }

      if let Some(text) = &entity.text {
        if text.value.is_empty() {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].text.value must not be empty"
          )));
        }
        if text.size.is_some_and(|s| {
          !s.is_finite() || s <= 0.0
        }) {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].text.size must be > 0"
          )));
        }
        if let Some(c) = text.color {
          validate_color(
            &format!(
              "scene.entities[{idx}].\
               text.color"
            ),
            &c
          )?;
        }
        if let Some(anchor) =
          text.anchor.as_deref()
        {
          validate_anchor(
            &format!(
              "scene.entities[{idx}].\
               text.anchor"
            ),
            anchor
          )?;
        }
        if let Some(align) =
          text.align.as_deref()
        {
          validate_align(
            &format!(
              "scene.entities[{idx}].\
               text.align"
            ),
            align
          )?;
        }
      }
    }

    Ok(())
  }
}

fn validate_color(
  path: &str,
  c: &ColorRgba
) -> Result<(), SceneError> {
  fn ok_chan(v: f32) -> bool {
    v.is_finite()
      && (0.0..=1.0).contains(&v)
  }

  if !ok_chan(c.r)
    || !ok_chan(c.g)
    || !ok_chan(c.b)
  {
    return Err(SceneError::Validate(
      format!(
        "{path} channels must be \
         finite and within [0, 1]"
      )
    ));
  }
  if let Some(a) = c.a {
    if !ok_chan(a) {
      return Err(SceneError::Validate(
        format!(
          "{path}.a must be finite \
           and within [0, 1]"
        )
      ));
    }
  }

  Ok(())
}

fn validate_anchor(
  path: &str,
  a: &str
) -> Result<(), SceneError> {
  let ok = matches!(
    a,
    "center"
      | "top_left"
      | "top"
      | "top_right"
      | "left"
      | "right"
      | "bottom_left"
      | "bottom"
      | "bottom_right"
  );
  if !ok {
    return Err(SceneError::Validate(
      format!(
        "{path} unsupported value \
         {a:?}"
      )
    ));
  }
  Ok(())
}

fn validate_align(
  path: &str,
  a: &str
) -> Result<(), SceneError> {
  let ok = matches!(
    a,
    "left" | "center" | "right"
  );
  if !ok {
    return Err(SceneError::Validate(
      format!(
        "{path} unsupported value \
         {a:?}"
      )
    ));
  }
  Ok(())
}
