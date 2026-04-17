#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf
};

use bevy::prelude::{
  Projection,
  *
};
use flatfekt_assets::resolve::{
  AssetResolveError,
  assets_root,
  bevy_load
};
use flatfekt_config::{
  ConfigError,
  RootConfig
};
use flatfekt_schema::{
  AssetRef,
  ColorRgba,
  SceneError,
  SceneFile,
  Transform2d
};
use tracing::instrument;

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

pub struct FlatfektRuntimePlugin;

impl Plugin for FlatfektRuntimePlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    app
      .add_message::<ResetScene>()
      .configure_sets(
        Startup,
        FlatfektSet::Instantiate
      )
      .configure_sets(
        Update,
        (
          FlatfektSet::Load,
          FlatfektSet::Instantiate
        )
      )
      .init_resource::<SpawnedEntities>(
      )
      .init_resource::<EntityMap>()
      .add_systems(
        Startup,
        instantiate_scene.in_set(
          FlatfektSet::Instantiate
        )
      )
      .add_systems(
        Update,
        (
          reset_scene_system,
          instantiate_scene
        )
          .chain()
          .run_if(bevy::ecs::schedule::common_conditions::on_message::<ResetScene>)
          .in_set(
            FlatfektSet::Instantiate
          )
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
    if let Some(zoom) = c.zoom {
      camera.insert(
        Projection::Orthographic(
          bevy::prelude::OrthographicProjection {
            scale: 1.0 / zoom,
            ..bevy::prelude::OrthographicProjection::default_2d()
          }
        )
      );
    }
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

  for ent in &scene.entities {
    let tf = transform_from_spec(
      ent.transform
    );

    if let Some(sprite_spec) =
      &ent.sprite
    {
      if let Some(e) = spawn_sprite(
        &mut commands,
        &assets,
        &assets_root.0,
        &ent.id,
        sprite_spec,
        tf
      ) {
        spawned.0.push(e);
        entity_map
          .0
          .entry(ent.id.clone())
          .or_default()
          .push(e);
      }
    }

    if let Some(text_spec) = &ent.text {
      let e = spawn_text(
        &mut commands,
        &assets,
        &assets_root.0,
        text_spec,
        tf
      );
      spawned.0.push(e);
      entity_map
        .0
        .entry(ent.id.clone())
        .or_default()
        .push(e);
    }
  }
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
  assets_root: &PathBuf,
  entity_id: &str,
  spec: &flatfekt_schema::SpriteSpec,
  tf: Transform
) -> Option<Entity> {
  let Some(path) = spec.image.as_path()
  else {
    tracing::error!(
      id = %entity_id,
      "sprite image is not a path"
    );
    return None;
  };

  let image_ref = AssetRef::Path {
    path: path.to_path_buf()
  };
  let handle = bevy_load::load_image(
    assets,
    assets_root,
    &image_ref
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
      let e = commands
        .spawn((sprite, tf))
        .id();
      Some(e)
    }
    | Err(err) => {
      tracing::error!(
        error = %err,
        id = %entity_id,
        "failed to load sprite image"
      );
      None
    }
  }
}

fn spawn_text(
  commands: &mut Commands,
  assets: &AssetServer,
  assets_root: &PathBuf,
  spec: &flatfekt_schema::TextSpec,
  tf: Transform
) -> Entity {
  let font_handle = spec
    .font
    .as_ref()
    .and_then(|a| a.as_path())
    .map(|p| {
      AssetRef::Path {
        path: p.to_path_buf()
      }
    })
    .and_then(|a| {
      bevy_load::load_font(
        assets,
        assets_root,
        &a
      )
      .ok()
    });

  let mut text_font = TextFont {
    font_size: spec
      .size
      .unwrap_or(24.0),
    ..default()
  };
  if let Some(h) = font_handle {
    text_font.font = h;
  }

  let justify =
    match spec.align.as_deref() {
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
  }

  entity.id()
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
    LoadError::Config {
      path: path.to_path_buf(),
      source
    }
  })
}

#[instrument(level = "info", skip_all, fields(path = %path.as_ref().display()))]
pub fn load_scene(
  path: impl AsRef<Path>
) -> Result<SceneFile, LoadError> {
  let path = path.as_ref();
  flatfekt_schema::SceneFile::load_from_path(path).map_err(|source| {
    LoadError::Scene {
      path: path.to_path_buf(),
      source
    }
  })
}
