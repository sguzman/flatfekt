#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::{
  Path,
  PathBuf
};

use serde::{
  Deserialize,
  Serialize
};
use tracing::instrument;

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
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
  schemars::JsonSchema,
  Debug,
  Clone,
  Copy,
  Serialize,
  Deserialize,
)]
pub struct ColorRgba {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: Option<f32>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct AssetPackSpec {
  pub name:     String,
  pub version:  String,
  pub root:     PathBuf,
  pub manifest: Option<PathBuf>
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

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SceneFile {
  pub scene: Scene
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct Scene {
  pub schema_version: String,
  pub resolution:
    Option<ResolutionSpec>,
  pub sequence:
    Option<Vec<SceneClipSpec>>,
  pub camera: Option<CameraSpec>,
  pub background:
    Option<BackgroundSpec>,
  pub playback: Option<PlaybackSpec>,
  pub effects: Option<Vec<EffectSpec>>,
  pub active_effect_id: Option<String>,
  pub defaults: Option<DefaultsSpec>,
  #[serde(default)]
  pub templates:
    std::collections::HashMap<
      String,
      EntitySpec
    >,
  #[serde(default)]
  pub timeline:
    Option<Vec<TimelineEvent>>,
  pub simulation: Option<SimRegionSpec>,
  pub interaction:
    Option<InteractionSpec>,
  pub export: Option<ExportSpec>,
  pub generators:
    Option<Vec<GeneratorSpec>>,
  pub baked: Option<std::path::PathBuf>,
  #[serde(default)]
  pub entities:       Vec<EntitySpec>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct GeneratorSpec {
  pub template:     String,
  pub count:        u32,
  pub distribution: DistributionSpec
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(
  rename_all = "snake_case",
  deny_unknown_fields
)]
pub enum DistributionSpec {
  Point {
    x: f32,
    y: f32
  },
  Box {
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32
  },
  Circle {
    x:      f32,
    y:      f32,
    radius: f32
  }
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ExportSpec {
  pub output_dir:
    Option<std::path::PathBuf>,
  pub screenshot: Option<bool>,
  pub recording:  Option<RecordingSpec>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct RecordingSpec {
  pub fps:      u32,
  pub duration: f32
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct InteractionSpec {
  pub actions: Vec<ActionBinding>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ActionBinding {
  pub action: String,
  pub keys:   Vec<String>,
  pub mouse:  Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SimRegionSpec {
  pub gravity:    Option<[f32; 2]>,
  pub bounds:     Option<[f32; 4]>,
  pub time_scale: Option<f32>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Copy,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ResolutionSpec {
  pub width:  u32,
  pub height: u32
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SceneClipSpec {
  pub path:          PathBuf,
  pub duration_secs: f32
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct EffectSpec {
  pub id:     String,
  pub wgsl:   Option<AssetRef>,
  pub glsl:   Option<AssetRef>,
  #[schemars(skip)]
  pub params: Option<toml::Value>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct PlaybackSpec {
  pub duration_secs:        Option<f32>,
  pub loop_mode: Option<String>,
  pub allow_user_input: Option<bool>,
  pub allow_scrub: Option<bool>,
  pub allow_rewind: Option<bool>,
  pub enable_introspection:
    Option<bool>,
  pub target_fps:           Option<u32>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct DefaultsSpec {
  pub text_font: Option<AssetRef>,
  pub text_font_fallback:
    Option<Vec<AssetRef>>,
  pub text_font_size:     Option<f32>,
  pub text_color: Option<ColorRgba>,
  pub sprite_anchor: Option<String>,
  pub text_anchor: Option<String>,
  pub text_align: Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct CameraSpec {
  pub x:            Option<f32>,
  pub y:            Option<f32>,
  pub zoom:         Option<f32>,
  pub scaling_mode: Option<String>,
  pub clear_color:  Option<ColorRgba>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct BackgroundSpec {
  pub clear_color: Option<ColorRgba>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct EntitySpec {
  pub id:          String,
  pub extends:     Option<String>,
  pub activation:
    Option<ActivationSpec>,
  pub tags:        Option<Vec<String>>,
  pub transform:   Option<Transform2d>,
  pub sprite:      Option<SpriteSpec>,
  pub text:        Option<TextSpec>,
  pub shape:       Option<ShapeSpec>,
  pub physics:     Option<PhysicsSpec>,
  pub collider:    Option<ColliderSpec>,
  pub particles:
    Option<ParticleSystemSpec>,
  pub interaction:
    Option<EntityInteractionSpec>,
  pub agent:       Option<AgentSpec>,
  pub script: Option<ScriptHookSpec>,
  pub grid:        Option<GridSpec>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct AgentSpec {
  pub kind:   String,
  #[schemars(skip)]
  pub params: Option<
    std::collections::HashMap<
      String,
      toml::Value
    >
  >
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ScriptHookSpec {
  pub on_spawn:  Option<String>,
  pub on_tick:   Option<String>,
  pub on_update: Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct GridSpec {
  pub width:         u32,
  pub height:        u32,
  pub cell_size:     f32,
  pub rule:          String,
  pub initial_state: Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct EntityInteractionSpec {
  pub selectable: Option<bool>,
  pub draggable:  Option<bool>,
  pub on_click:   Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ActivationSpec {
  pub features: Option<Vec<String>>,
  pub platform: Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(tag = "op")]
#[serde(rename_all = "lowercase")]
pub enum ScenePatch {
  Add {
    entity: EntitySpec
  },
  Remove {
    entity_id: String
  },
  Update {
    entity_id: String,
    patch:     EntityPatch
  }
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct EntityPatch {
  pub tags:      Option<Vec<String>>,
  pub transform: Option<Transform2d>,
  pub sprite:    Option<SpriteSpec>,
  pub text:      Option<TextSpec>,
  pub shape:     Option<ShapeSpec>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TimelineEvent {
  pub time:    f32,
  pub action:  String,
  pub target:  Option<String>,
  #[schemars(skip)]
  pub payload: Option<toml::Value>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Copy,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct Transform2d {
  pub x:        f32,
  pub y:        f32,
  pub rotation: Option<f32>,
  pub scale:    Option<f32>,
  pub z:        Option<f32>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SpriteSpec {
  pub image:     AssetRef,
  pub width:     Option<f32>,
  pub height:    Option<f32>,
  pub anchor:    Option<String>,
  pub tint:      Option<ColorRgba>,
  pub opacity:   Option<f32>,
  pub effect_id: Option<String>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TextSpan {
  pub value:  String,
  pub font:   Option<AssetRef>,
  pub size:   Option<f32>,
  pub color:  Option<ColorRgba>,
  pub weight: Option<String>,
  pub italic: Option<bool>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TextSpec {
  pub value:   Option<String>,
  pub spans:   Option<Vec<TextSpan>>,
  pub font:    Option<AssetRef>,
  pub size:    Option<f32>,
  pub color:   Option<ColorRgba>,
  pub anchor:  Option<String>,
  pub align:   Option<String>,
  pub effects:
    Option<Vec<TextEffectSpec>>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TextEffectSpec {
  pub kind:   String,
  pub color:  Option<ColorRgba>,
  pub offset: Option<[f32; 2]>,
  pub radius: Option<f32>,
  pub shader: Option<AssetRef>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ShapeSpec {
  pub kind:  String,
  pub color: Option<ColorRgba>,

  pub width:  Option<f32>,
  pub height: Option<f32>,

  pub radius: Option<f32>,

  pub sides: Option<u32>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct PhysicsSpec {
  pub body_type:       String,
  pub mass:            Option<f32>,
  pub friction:        Option<f32>,
  pub restitution:     Option<f32>,
  pub linear_damping:  Option<f32>,
  pub angular_damping: Option<f32>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ColliderSpec {
  pub kind:   String,
  pub size:   Option<[f32; 2]>,
  pub radius: Option<f32>,
  pub sensor: Option<bool>
}

#[derive(
  schemars::JsonSchema,
  Debug,
  Clone,
  Serialize,
  Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ParticleSystemSpec {
  pub emission_rate: f32,
  pub lifetime:      f32,
  pub velocity_min:  [f32; 2],
  pub velocity_max:  [f32; 2],
  pub max_particles: u32
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
    if self
      .schema_version
      .trim()
      .is_empty()
    {
      return Err(SceneError::Validate(
        "scene.schema_version must \
         not be empty"
          .to_owned()
      ));
    }
    if self.schema_version != "0.1" {
      return Err(SceneError::Validate(
        format!(
          "scene.schema_version \
           unsupported value {:?} \
           (expected \"0.1\")",
          self.schema_version
        )
      ));
    }

    let has_sequence = self
      .sequence
      .as_ref()
      .is_some_and(|s| !s.is_empty());
    if self.entities.is_empty()
      && !has_sequence
    {
      return Err(SceneError::Validate(
        "scene.entities must not be \
         empty (unless scene.sequence \
         is present)"
          .to_owned()
      ));
    }

    if let Some(res) = self.resolution {
      if res.width == 0
        || res.height == 0
      {
        return Err(
          SceneError::Validate(
            "scene.resolution.width/\
             height must be >= 1"
              .to_owned()
          )
        );
      }
    }

    if let Some(seq) = &self.sequence {
      for (idx, clip) in
        seq.iter().enumerate()
      {
        if clip.duration_secs <= 0.0
          || !clip
            .duration_secs
            .is_finite()
        {
          return Err(SceneError::Validate(
            format!(
              "scene.sequence[{idx}].duration_secs must be finite and > 0"
            )
          ));
        }
        let p =
          clip.path.to_string_lossy();
        if p.trim().is_empty() {
          return Err(SceneError::Validate(
            format!(
              "scene.sequence[{idx}].path must not be empty"
            )
          ));
        }
      }
    }

    if let Some(d) = &self.defaults {
      if let Some(sz) = d.text_font_size
      {
        if !sz.is_finite() || sz <= 0.0
        {
          return Err(
            SceneError::Validate(
              "scene.defaults.\
               text_font_size must be \
               > 0"
                .to_owned()
            )
          );
        }
      }
      if let Some(c) = d.text_color {
        validate_color(
          "scene.defaults.text_color",
          &c
        )?;
      }
      if let Some(a) =
        d.sprite_anchor.as_deref()
      {
        validate_anchor(
          "scene.defaults.\
           sprite_anchor",
          a
        )?;
      }
      if let Some(a) =
        d.text_anchor.as_deref()
      {
        validate_anchor(
          "scene.defaults.text_anchor",
          a
        )?;
      }
      if let Some(a) =
        d.text_align.as_deref()
      {
        validate_align(
          "scene.defaults.text_align",
          a
        )?;
      }
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
      if let Some(mode) =
        camera.scaling_mode.as_deref()
      {
        validate_scaling_mode(
          "scene.camera.scaling_mode",
          mode
        )?;
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

    if let Some(pb) = &self.playback {
      if let Some(dur) =
        pb.duration_secs
      {
        if !dur.is_finite()
          || dur <= 0.0
        {
          return Err(
            SceneError::Validate(
              "scene.playback.\
               duration_secs must be \
               > 0"
                .to_owned()
            )
          );
        }
      }
      if let Some(mode) =
        pb.loop_mode.as_deref()
      {
        let ok = matches!(
          mode,
          "stop" | "loop"
        );
        if !ok {
          return Err(
            SceneError::Validate(
              format!(
                "scene.playback.\
                 loop_mode unsupported \
                 value {:?} (expected \
                 \"stop\" or \"loop\")",
                mode
              )
            )
          );
        }
      }
      if pb
        .allow_rewind
        .unwrap_or(false)
        && !pb
          .allow_scrub
          .unwrap_or(false)
      {
        return Err(
          SceneError::Validate(
            "scene.playback.\
             allow_rewind requires \
             scene.playback.\
             allow_scrub"
              .to_owned()
          )
        );
      }
    }

    if let Some(effects) = &self.effects
    {
      let mut ids =
        std::collections::HashSet::<&str>::new();
      for (eidx, e) in
        effects.iter().enumerate()
      {
        if e.id.trim().is_empty() {
          return Err(SceneError::Validate(format!(
            "scene.effects[{eidx}].id must not be empty"
          )));
        }
        if !ids.insert(e.id.as_str()) {
          return Err(
            SceneError::Validate(
              format!(
                "duplicate effect id \
                 {:?} (scene.\
                 effects[{eidx}])",
                e.id
              )
            )
          );
        }
        if e.wgsl.is_none() && e.glsl.is_none() {
          return Err(SceneError::Validate(format!(
            "scene.effects[{eidx}] must have either wgsl or glsl specified"
          )));
        }
        if e.wgsl.is_some() && e.glsl.is_some() {
          return Err(SceneError::Validate(format!(
            "scene.effects[{eidx}] cannot have both wgsl and glsl specified"
          )));
        }

        let shader_ref = e.wgsl.as_ref().or(e.glsl.as_ref());
        if let Some(shader) = shader_ref {
          match shader {
            | AssetRef::String(s)
              if s.trim().is_empty() =>
            {
              return Err(SceneError::Validate(format!(
                "scene.effects[{eidx}] shader path must not be empty"
              )));
            }
            | AssetRef::Path {
              path
            } if path
              .as_os_str()
              .is_empty() =>
            {
              return Err(SceneError::Validate(format!(
                "scene.effects[{eidx}] shader path must not be empty"
              )));
            }
            | AssetRef::Id {
              id
            } if id.trim().is_empty() => {
              return Err(SceneError::Validate(format!(
                "scene.effects[{eidx}] shader id must not be empty"
              )));
            }
            | _ => {}
          }
        }
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
        && entity.shape.is_none()
        && entity.particles.is_none()
        && entity.grid.is_none()
      {
        return Err(
          SceneError::Validate(
            format!(
              "scene.entities[{idx}] \
               ({:?}) must define at \
               least one of [sprite, \
               text, shape, \
               particles, grid]",
              entity.id
            )
          )
        );
      }

      if let Some(tags) = &entity.tags {
        for (tidx, t) in
          tags.iter().enumerate()
        {
          if t.trim().is_empty() {
            return Err(SceneError::Validate(format!(
              "scene.entities[{idx}].tags[{tidx}] must not be empty"
            )));
          }
        }
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
        match &sprite.image {
          | AssetRef::Id {
            id
          } => {
            if id.trim().is_empty() {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].sprite.image id must not be empty"
              )));
            }
          }
          | AssetRef::Path {
            path
          } => {
            if path
              .as_os_str()
              .is_empty()
            {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].sprite.image must not be empty"
              )));
            }
          }
          | AssetRef::String(s) => {
            if s.trim().is_empty() {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].sprite.image must not be empty"
              )));
            }
          }
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
        if let Some(tint) = sprite.tint
        {
          validate_color(
            &format!(
              "scene.entities[{idx}].\
               sprite.tint"
            ),
            &tint
          )?;
        }
        if let Some(opacity) =
          sprite.opacity
        {
          if !opacity.is_finite()
            || !(0.0..=1.0)
              .contains(&opacity)
          {
            return Err(SceneError::Validate(format!(
              "scene.entities[{idx}].sprite.opacity must be within [0, 1]"
            )));
          }
        }
        if let Some(effect_id) =
          sprite.effect_id.as_deref()
        {
          if effect_id.trim().is_empty()
          {
            return Err(SceneError::Validate(format!(
              "scene.entities[{idx}].sprite.effect_id must not be empty"
            )));
          }
          if let Some(effects) =
            &self.effects
          {
            if !effects.iter().any(
              |e| e.id == effect_id
            ) {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].sprite.effect_id references unknown effect id {:?}",
                effect_id
              )));
            }
          } else {
            return Err(SceneError::Validate(format!(
              "scene.entities[{idx}].sprite.effect_id references effect id {:?} but scene.effects is not defined",
              effect_id
            )));
          }
        }
      }

      if let Some(text) = &entity.text {
        if text.value.is_none()
          && text.spans.is_none()
        {
          return Err(SceneError::Validate(
            format!(
              "scene.entities[{idx}].text \
               must have either value or \
               spans"
            )
          ));
        }
        if let Some(v) = &text.value {
          if v.is_empty() {
            return Err(SceneError::Validate(
              format!(
                "scene.entities[{idx}].text.\
                 value must not be empty"
              )
            ));
          }
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

        if let Some(font) = &text.font {
          if let AssetRef::Id {
            id
          } = font
          {
            if id.trim().is_empty() {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].text.font id must not be empty"
              )));
            }
          }
        }
      }

      if let Some(shape) = &entity.shape
      {
        let kind = shape.kind.as_str();
        let ok = matches!(
          kind,
          "rect" | "circle" | "polygon"
        );
        if !ok {
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].shape.kind unsupported value {kind:?}"
          )));
        }
        if let Some(c) = shape.color {
          validate_color(
            &format!(
              "scene.entities[{idx}].\
               shape.color"
            ),
            &c
          )?;
        }
        match kind {
          | "rect" => {
            let Some(w) = shape.width
            else {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.width is required for rect"
              )));
            };
            let Some(h) = shape.height
            else {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.height is required for rect"
              )));
            };
            if !w.is_finite()
              || w <= 0.0
            {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.width must be > 0"
              )));
            }
            if !h.is_finite()
              || h <= 0.0
            {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.height must be > 0"
              )));
            }
          }
          | "circle" => {
            let Some(r) = shape.radius
            else {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.radius is required for circle"
              )));
            };
            if !r.is_finite()
              || r <= 0.0
            {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.radius must be > 0"
              )));
            }
          }
          | "polygon" => {
            let Some(r) = shape.radius
            else {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.radius is required for polygon"
              )));
            };
            let Some(sides) =
              shape.sides
            else {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.sides is required for polygon"
              )));
            };
            if !r.is_finite()
              || r <= 0.0
            {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.radius must be > 0"
              )));
            }
            if sides < 3 {
              return Err(SceneError::Validate(format!(
                "scene.entities[{idx}].shape.sides must be >= 3"
              )));
            }
          }
          | _ => {}
        }
      }
    }
    // Validate extends references and
    // did you mean
    for (idx, entity) in
      self.entities.iter().enumerate()
    {
      if let Some(ext) = &entity.extends
      {
        if !self
          .templates
          .contains_key(ext)
        {
          let mut best_match = None;
          let mut best_dist =
            usize::MAX;
          for tpl in
            self.templates.keys()
          {
            let dist =
              strsim::levenshtein(
                ext, tpl
              );
            if dist < best_dist
              && dist <= 3
            {
              best_dist = dist;
              best_match = Some(tpl);
            }
          }
          let did_you_mean =
            if let Some(m) = best_match
            {
              format!(
                " (did you mean {:?}?)",
                m
              )
            } else {
              String::new()
            };
          return Err(SceneError::Validate(format!(
            "scene.entities[{idx}].extends references unknown template {:?}{did_you_mean}",
            ext
          )));
        }
      }
    }

    if let Some(timeline) =
      &self.timeline
    {
      for (idx, ev) in
        timeline.iter().enumerate()
      {
        if let Some(target) = &ev.target
        {
          if !ids
            .contains(target.as_str())
            && !self
              .templates
              .contains_key(target)
          {
            let mut best_match = None;
            let mut best_dist =
              usize::MAX;
            for tpl in self
              .templates
              .keys()
              .map(|s| s.as_str())
              .chain(
                ids.iter().map(|s| *s)
              )
            {
              let dist =
                strsim::levenshtein(
                  target, tpl
                );
              if dist < best_dist
                && dist <= 3
              {
                best_dist = dist;
                best_match = Some(tpl);
              }
            }
            let did_you_mean =
              if let Some(m) =
                best_match
              {
                format!(
                  " (did you mean \
                   {:?}?)",
                  m
                )
              } else {
                String::new()
              };
            return Err(SceneError::Validate(format!(
              "scene.timeline[{idx}].target references unknown ID {:?}{did_you_mean}",
              target
            )));
          }
        }
      }
    }

    Ok(())
  }
}

impl Scene {
  pub fn entities_sorted_by_id(
    &self
  ) -> Vec<&EntitySpec> {
    let mut out: Vec<&EntitySpec> =
      self.entities.iter().collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
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

fn validate_scaling_mode(
  path: &str,
  m: &str
) -> Result<(), SceneError> {
  let ok = matches!(
    m,
    "window_size"
      | "fixed_horizontal"
      | "fixed_vertical"
      | "fixed"
  );
  if !ok {
    return Err(SceneError::Validate(
      format!(
        "{path} unsupported value \
         {m:?}"
      )
    ));
  }
  Ok(())
}
