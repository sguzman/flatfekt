#![forbid(unsafe_code)]

use std::path::PathBuf;

use bevy::prelude::*;
use flatfekt_assets::resolve::{
  assets_root,
  bevy_load
};
use flatfekt_config::RootConfig;
use flatfekt_schema::{
  AssetRef,
  ColorRgba,
  SceneError,
  SceneFile
};
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
  #[error("asset resolution failed: {0}")]
  Assets(
    #[from]
    flatfekt_assets::resolve::AssetResolveError
  ),

  #[error("scene error: {0}")]
  Scene(
    #[from]
    SceneError
  )
}

#[derive(Resource, Clone)]
pub struct ConfigRes(pub RootConfig);

#[derive(Resource, Clone)]
pub struct SceneRes(pub SceneFile);

#[derive(Resource, Clone)]
pub struct AssetsRootRes(pub PathBuf);

pub struct FlatfektRuntimePlugin;

impl Plugin for FlatfektRuntimePlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    app.add_systems(
      Startup,
      instantiate_scene
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
  assets_root: Res<AssetsRootRes>
) {
  let _ = &cfg.0;
  let scene = &scene.0.scene;

  commands.spawn(Camera2d);

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
      spawn_sprite(
        &mut commands,
        &assets,
        &assets_root.0,
        &ent.id,
        sprite_spec,
        tf
      );
    }

    if let Some(text_spec) = &ent.text {
      spawn_text(
        &mut commands,
        &assets,
        &assets_root.0,
        text_spec,
        tf
      );
    }
  }
}

fn transform_from_spec(
  t: Option<
    flatfekt_schema::Transform2d
  >
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
) {
  let Some(path) = spec.image.as_path()
  else {
    tracing::error!(
      id = %entity_id,
      "sprite image is not a path"
    );
    return;
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
      // z-order is carried via
      // Transform.translation.z
      commands.spawn((sprite, tf));
    }
    | Err(err) => {
      tracing::error!(
        error = %err,
        id = %entity_id,
        "failed to load sprite image"
      );
    }
  }
}

fn spawn_text(
  commands: &mut Commands,
  assets: &AssetServer,
  assets_root: &PathBuf,
  spec: &flatfekt_schema::TextSpec,
  tf: Transform
) {
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
