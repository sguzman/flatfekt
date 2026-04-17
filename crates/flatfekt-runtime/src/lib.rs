#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf
};

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_camera::ScalingMode;
use bevy_mesh::Mesh2d;
use flatfekt_assets::resolve::{
  AssetCache,
  AssetResolveError,
  assets_root,
  bevy_load
};
use flatfekt_config::{
  ConfigError,
  RootConfig
};
use flatfekt_schema::{
  ColorRgba,
  SceneError,
  SceneFile,
  Transform2d
};
use tracing::instrument;

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
)]
pub struct RenderSortKey {
  pub layer:   u32,
  pub z_bits:  u32,
  pub kind:    u8,
  pub id_hash: u64
}

pub fn render_sort_key(
  id: &str,
  z: f32,
  kind: RenderKind
) -> RenderSortKey {
  RenderSortKey {
    layer:   0,
    z_bits:  z.to_bits(),
    kind:    kind as u8,
    id_hash: fxhash64(id.as_bytes())
  }
}

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum RenderKind {
  Shape  = 0,
  Sprite = 1,
  Text   = 2
}

fn fxhash64(bytes: &[u8]) -> u64 {
  // Simple stable hash for tie-break
  // only (not crypto).
  let mut hash: u64 =
    0xcbf29ce484222325;
  for b in bytes {
    hash ^= *b as u64;
    hash =
      hash.wrapping_mul(0x100000001b3);
  }
  hash
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
  #[error(
    "asset resolution failed: {0}"
  )]
  Assets(#[from] AssetResolveError),

  #[error("scene error: {0}")]
  Scene(#[from] SceneError)
}

#[derive(Resource, Clone)]
pub struct ConfigRes(pub RootConfig);

#[derive(Resource, Clone)]
pub struct SceneRes(pub SceneFile);

#[derive(Resource, Clone)]
pub struct AssetsRootRes(pub PathBuf);

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  SystemSet,
)]
pub enum FlatfektSet {
  Load,
  Instantiate
}

#[derive(Resource, Default)]
pub struct SpawnedEntities(
  pub Vec<Entity>
);

#[derive(Resource, Default)]
pub struct EntityMap(
  pub HashMap<String, Vec<Entity>>
);

#[derive(Message, Default)]
pub struct ResetScene;

#[derive(Resource, Default)]
pub struct AssetsCacheRes(
  pub AssetCache
);

pub struct FlatfektRuntimePlugin;

impl Plugin for FlatfektRuntimePlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    app.add_message::<ResetScene>()
      .configure_sets(Startup, FlatfektSet::Instantiate)
      .configure_sets(
        Update,
        (
          FlatfektSet::Load,
          FlatfektSet::Instantiate
        )
      )
      .init_resource::<SpawnedEntities>()
      .init_resource::<EntityMap>()
      .init_resource::<AssetsCacheRes>()
      .add_systems(
        Startup,
        instantiate_scene.in_set(FlatfektSet::Instantiate)
      )
      .add_systems(
        Update,
        (reset_scene_system, instantiate_scene)
          .chain()
          .run_if(bevy::ecs::schedule::common_conditions::on_message::<ResetScene>)
          .in_set(FlatfektSet::Instantiate)
      );
  }
}

pub fn run_bevy(
  cfg: RootConfig,
  scene: SceneFile
) -> Result<(), RuntimeError> {
  let root = assets_root(&cfg)?;

  App::new()
    .insert_resource(ConfigRes(cfg))
    .insert_resource(SceneRes(scene))
    .insert_resource(AssetsRootRes(
      root
    ))
    .add_plugins(DefaultPlugins)
    .add_plugins(FlatfektRuntimePlugin)
    .run();

  Ok(())
}

#[instrument(level = "info", skip_all)]
fn instantiate_scene(
  mut commands: Commands,
  assets: Res<AssetServer>,
  cfg: Res<ConfigRes>,
  scene: Res<SceneRes>,
  assets_root: Res<AssetsRootRes>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<
    Assets<ColorMaterial>
  >,
  mut asset_cache: ResMut<
    AssetsCacheRes
  >,
  mut spawned: ResMut<SpawnedEntities>,
  mut entity_map: ResMut<EntityMap>
) {
  let _ = &cfg.0;
  let scene = &scene.0.scene;

  spawned.0.clear();
  entity_map.0.clear();

  let mut camera =
    commands.spawn(Camera2d);
  if let Some(c) = &scene.camera {
    let x = c.x.unwrap_or(0.0);
    let y = c.y.unwrap_or(0.0);
    camera.insert(
      Transform::from_translation(
        Vec3::new(x, y, 0.0)
      )
    );

    let zoom = c.zoom.unwrap_or(1.0);
    let mut ortho = OrthographicProjection::default_2d();
    ortho.scale = 1.0 / zoom;
    if let Some(mode) =
      c.scaling_mode.as_deref()
    {
      ortho.scaling_mode =
        scaling_mode_from_str(mode);
    }
    camera.insert(
      Projection::Orthographic(ortho)
    );
  }

  let clear_color = scene
    .background
    .as_ref()
    .and_then(|b| b.clear_color)
    .or_else(|| {
      scene
        .camera
        .as_ref()
        .and_then(|c| c.clear_color)
    })
    .map(color_from_rgba);

  if let Some(cc) = clear_color {
    commands
      .insert_resource(ClearColor(cc));
  }

  let mut plan: Vec<SpawnOp> =
    Vec::new();
  for ent in &scene.entities {
    let tf = transform_from_spec(
      ent.transform
    );
    if let Some(shape) = &ent.shape {
      plan.push(SpawnOp {
        id: &ent.id,
        kind: RenderKind::Shape,
        tf,
        shape: Some(shape),
        sprite: None,
        text: None
      });
    }
    if let Some(sprite) = &ent.sprite {
      plan.push(SpawnOp {
        id: &ent.id,
        kind: RenderKind::Sprite,
        tf,
        shape: None,
        sprite: Some(sprite),
        text: None
      });
    }
    if let Some(text) = &ent.text {
      plan.push(SpawnOp {
        id: &ent.id,
        kind: RenderKind::Text,
        tf,
        shape: None,
        sprite: None,
        text: Some(text)
      });
    }
  }

  plan.sort_by_key(|op| {
    let z = op.tf.translation.z;
    (
      0u32,
      z.to_bits(),
      op.kind as u8,
      op.id
    )
  });

  for op in plan {
    let e = match op.kind {
      | RenderKind::Shape => {
        spawn_shape(
          &mut commands,
          &mut meshes,
          &mut materials,
          op.id,
          scene.defaults.as_ref(),
          op.shape.unwrap(),
          op.tf
        )
      }
      | RenderKind::Sprite => {
        spawn_sprite(
          &mut commands,
          &assets,
          &cfg.0,
          &assets_root.0,
          &mut asset_cache.0,
          op.id,
          scene.defaults.as_ref(),
          op.sprite.unwrap(),
          op.tf
        )
      }
      | RenderKind::Text => {
        Some(spawn_text(
          &mut commands,
          &assets,
          &cfg.0,
          &assets_root.0,
          &mut asset_cache.0,
          scene.defaults.as_ref(),
          op.text.unwrap(),
          op.tf
        ))
      }
    };

    if let Some(e) = e {
      spawned.0.push(e);
      entity_map
        .0
        .entry(op.id.to_owned())
        .or_default()
        .push(e);
    }
  }
}

struct SpawnOp<'a> {
  id:     &'a str,
  kind:   RenderKind,
  tf:     Transform,
  shape: Option<
    &'a flatfekt_schema::ShapeSpec
  >,
  sprite: Option<
    &'a flatfekt_schema::SpriteSpec
  >,
  text: Option<
    &'a flatfekt_schema::TextSpec
  >
}

fn transform_from_spec(
  t: Option<Transform2d>
) -> Transform {
  let mut tf = Transform::default();
  if let Some(t) = t {
    tf.translation = Vec3::new(
      t.x,
      t.y,
      t.z.unwrap_or(0.0)
    );
    if let Some(r) = t.rotation {
      tf.rotation =
        Quat::from_rotation_z(r);
    }
    if let Some(s) = t.scale {
      tf.scale = Vec3::splat(s);
    }
  }
  tf
}

fn spawn_sprite(
  commands: &mut Commands,
  assets: &AssetServer,
  cfg: &flatfekt_config::RootConfig,
  assets_root: &PathBuf,
  asset_cache: &mut AssetCache,
  entity_id: &str,
  defaults: Option<
    &flatfekt_schema::DefaultsSpec
  >,
  spec: &flatfekt_schema::SpriteSpec,
  tf: Transform
) -> Option<Entity> {
  let handle =
    bevy_load::load_image_cached(
      asset_cache,
      assets,
      cfg,
      assets_root,
      &spec.image
    );

  match handle {
    | Ok(handle) => {
      let mut sprite =
        Sprite::from_image(handle);
      if let (Some(w), Some(h)) =
        (spec.width, spec.height)
      {
        sprite.custom_size =
          Some(Vec2::new(w, h));
      }
      let mut entity =
        commands.spawn((sprite, tf));
      if let Some(anchor) = spec
        .anchor
        .as_deref()
        .or_else(|| {
          defaults.and_then(|d| {
            d.sprite_anchor.as_deref()
          })
        })
      {
        entity.insert(anchor_from_str(
          anchor
        ));
      }
      Some(entity.id())
    }
    | Err(err) => {
      tracing::error!(error = %err, id = %entity_id, "failed to load sprite image");
      None
    }
  }
}

fn spawn_text(
  commands: &mut Commands,
  assets: &AssetServer,
  cfg: &flatfekt_config::RootConfig,
  assets_root: &PathBuf,
  asset_cache: &mut AssetCache,
  defaults: Option<
    &flatfekt_schema::DefaultsSpec
  >,
  spec: &flatfekt_schema::TextSpec,
  tf: Transform
) -> Entity {
  let font_handle =
    spec.font.as_ref().and_then(|a| {
      bevy_load::load_font_cached(
        asset_cache,
        assets,
        cfg,
        assets_root,
        a
      )
      .ok()
    });

  let mut text_font = TextFont {
    font_size: spec
      .size
      .or_else(|| {
        defaults.and_then(|d| {
          d.text_font_size
        })
      })
      .unwrap_or(24.0),
    ..default()
  };
  if let Some(h) = font_handle {
    text_font.font = h;
  }

  let align = spec
    .align
    .as_deref()
    .or_else(|| {
      defaults.and_then(|d| {
        d.text_align.as_deref()
      })
    });
  let justify = match align {
    | Some("left") => Justify::Left,
    | Some("right") => Justify::Right,
    | _ => Justify::Center
  };

  let mut entity = commands.spawn((
    Text2d::new(spec.value.clone()),
    text_font,
    TextLayout::new_with_justify(
      justify
    ),
    tf
  ));

  if let Some(c) = spec.color {
    entity.insert(TextColor(
      color_from_rgba(c)
    ));
  } else if let Some(c) =
    defaults.and_then(|d| d.text_color)
  {
    entity.insert(TextColor(
      color_from_rgba(c)
    ));
  }

  if let Some(anchor) = spec
    .anchor
    .as_deref()
    .or_else(|| {
      defaults.and_then(|d| {
        d.text_anchor.as_deref()
      })
    })
  {
    entity
      .insert(anchor_from_str(anchor));
  }

  entity.id()
}

fn spawn_shape(
  commands: &mut Commands,
  meshes: &mut Assets<Mesh>,
  mats: &mut Assets<ColorMaterial>,
  entity_id: &str,
  defaults: Option<
    &flatfekt_schema::DefaultsSpec
  >,
  spec: &flatfekt_schema::ShapeSpec,
  tf: Transform
) -> Option<Entity> {
  let color = spec
    .color
    .map(color_from_rgba)
    .or_else(|| {
      defaults
        .and_then(|d| d.text_color)
        .map(color_from_rgba)
    })
    .unwrap_or(Color::WHITE);

  match spec.kind.as_str() {
    | "rect" => {
      let w = spec.width?;
      let h = spec.height?;
      let sprite = Sprite::from_color(
        color,
        Vec2::new(w, h)
      );
      Some(
        commands
          .spawn((sprite, tf))
          .id()
      )
    }
    | "circle" => {
      let r = spec.radius?;
      let mesh = meshes.add(
        Mesh::from(Circle::new(r))
      );
      let mat = mats.add(color);
      Some(
        commands
          .spawn((
            Mesh2d(mesh),
            bevy::prelude::MeshMaterial2d(mat),
            tf,
            Name::new(format!(
              "{entity_id}-circle"
            ))
          ))
          .id()
      )
    }
    | "polygon" => {
      let r = spec.radius?;
      let sides = spec.sides?;
      let mesh =
        meshes.add(Mesh::from(
          RegularPolygon::new(r, sides)
        ));
      let mat = mats.add(color);
      Some(
        commands
          .spawn((
            Mesh2d(mesh),
            bevy::prelude::MeshMaterial2d(mat),
            tf,
            Name::new(format!(
              "{entity_id}-polygon"
            ))
          ))
          .id()
      )
    }
    | _ => {
      tracing::error!(id = %entity_id, kind = %spec.kind, "unknown shape kind");
      None
    }
  }
}

fn color_from_rgba(
  c: ColorRgba
) -> Color {
  Color::srgba(
    c.r,
    c.g,
    c.b,
    c.a.unwrap_or(1.0)
  )
}

fn anchor_from_str(a: &str) -> Anchor {
  match a {
    | "top_left" => Anchor::TOP_LEFT,
    | "top" => Anchor::TOP_CENTER,
    | "top_right" => Anchor::TOP_RIGHT,
    | "left" => Anchor::CENTER_LEFT,
    | "right" => Anchor::CENTER_RIGHT,
    | "bottom_left" => {
      Anchor::BOTTOM_LEFT
    }
    | "bottom" => Anchor::BOTTOM_CENTER,
    | "bottom_right" => {
      Anchor::BOTTOM_RIGHT
    }
    | _ => Anchor::CENTER
  }
}

fn scaling_mode_from_str(
  m: &str
) -> ScalingMode {
  match m {
    | "fixed_horizontal" => {
      ScalingMode::FixedHorizontal {
        viewport_width: 1080.0
      }
    }
    | "fixed_vertical" => {
      ScalingMode::FixedVertical {
        viewport_height: 720.0
      }
    }
    | "fixed" => {
      ScalingMode::Fixed {
        width:  1280.0,
        height: 720.0
      }
    }
    | _ => ScalingMode::WindowSize
  }
}

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
  #[error("unknown entity id: {0}")]
  UnknownId(String),

  #[error(
    "entity id has no runtime \
     entities: {0}"
  )]
  EmptyId(String)
}

pub fn lookup_entities<'a>(
  map: &'a EntityMap,
  id: &str
) -> Result<&'a [Entity], LookupError> {
  let Some(list) = map.0.get(id) else {
    tracing::warn!(
      id,
      "entity id lookup failed"
    );
    return Err(LookupError::UnknownId(
      id.to_owned()
    ));
  };
  if list.is_empty() {
    tracing::warn!(
      id,
      "entity id resolved to empty \
       list"
    );
    return Err(LookupError::EmptyId(
      id.to_owned()
    ));
  }
  Ok(list.as_slice())
}

#[instrument(level = "info", skip_all)]
fn reset_scene_system(
  mut commands: Commands,
  spawned: Res<SpawnedEntities>
) {
  for e in &spawned.0 {
    commands.entity(*e).despawn();
  }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
  #[error(
    "failed to load config at {path}: \
     {source}"
  )]
  Config {
    path:   PathBuf,
    #[source]
    source: ConfigError
  },

  #[error(
    "failed to load scene at {path}: \
     {source}"
  )]
  Scene {
    path:   PathBuf,
    #[source]
    source: SceneError
  }
}

#[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
pub fn load_config(
  path: impl AsRef<Path>
) -> Result<RootConfig, LoadError> {
  let path = path.as_ref();
  flatfekt_config::RootConfig::load_from_path(path).map_err(|source| {
    LoadError::Config { path: path.to_path_buf(), source }
  })
}

#[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
pub fn load_scene(
  path: impl AsRef<Path>
) -> Result<SceneFile, LoadError> {
  let path = path.as_ref();
  flatfekt_schema::SceneFile::load_from_path(path).map_err(|source| {
    LoadError::Scene { path: path.to_path_buf(), source }
  })
}
