use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
  AsBindGroup,
  Extent3d,
  ShaderType,
  TextureDimension,
  TextureFormat,
  RenderPipelineDescriptor,
  SpecializedMeshPipelineError
};
use bevy_camera::RenderTarget;
use bevy_mesh::Mesh2d;
use bevy_shader::ShaderRef;
use bevy_sprite_render::{
  Material2d,
  Material2dKey,
  Material2dPlugin,
  MeshMaterial2d
};
use tracing::instrument;

use crate::{
  AssetsRootRes,
  ConfigRes,
  SceneRes
};

#[derive(Resource, Debug, Clone)]
pub struct EffectsConfigRes {
  pub enabled:          bool,
  pub global_effect_id: Option<String>,
  pub rtt_enabled:      bool,
  pub rtt_scale:        f32
}

#[derive(Resource, Debug, Clone)]
pub struct RenderToTextureRes {
  pub image: Handle<Image>
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct PostProcessKey {
  pub shader: Handle<Shader>
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(PostProcessKey)]
pub struct PostProcessMaterial {
  #[texture(0)]
  #[sampler(1)]
  pub source: Handle<Image>,
  #[uniform(2)]
  pub params: PostProcessParams,
  pub shader: Handle<Shader>
}

impl From<&PostProcessMaterial> for PostProcessKey {
  fn from(m: &PostProcessMaterial) -> Self {
    Self {
      shader: m.shader.clone()
    }
  }
}

#[derive(
  ShaderType, Debug, Clone, Copy,
)]
pub struct PostProcessParams {
  pub intensity: f32
}

impl Material2d
  for PostProcessMaterial
{
  fn fragment_shader() -> ShaderRef {
    "flatfekt/shaders/postprocess.wgsl"
      .into()
  }

  fn specialize(
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayoutRef,
    key: Material2dKey<Self>,
  ) -> Result<(), SpecializedMeshPipelineError> {
    descriptor.fragment.as_mut().unwrap().shader =
      key.bind_group_data.shader.clone();
    Ok(())
  }
}

pub struct FlatfektEffectsPlugin;

impl Plugin for FlatfektEffectsPlugin {
  fn build(
    &self,
    app: &mut App
  ) {
    app
      .add_plugins(Material2dPlugin::<
        PostProcessMaterial
      >::default())
      .add_systems(
        Startup,
        (
          init_effects_config,
          init_rtt_image
        )
          .chain()
      )
      .add_systems(
        Update,
        apply_camera_targets
      );
  }
}

#[instrument(level = "info", skip_all)]
fn init_effects_config(
  cfg: Res<ConfigRes>,
  mut commands: Commands
) {
  let cfg = &cfg.0;
  let res = EffectsConfigRes {
    enabled:          cfg
      .render_effects_enabled(),
    global_effect_id: cfg
      .render_effects_global_id()
      .map(|s| s.to_owned()),
    rtt_enabled:      cfg
      .render_effects_rtt_enabled(),
    rtt_scale:        cfg
      .render_effects_rtt_scale()
  };
  commands.insert_resource(res);
}

#[instrument(level = "info", skip_all)]
fn init_rtt_image(
  effects: Res<EffectsConfigRes>,
  mut images: ResMut<Assets<Image>>,
  mut commands: Commands
) {
  if !effects.enabled
    || !effects.rtt_enabled
  {
    return;
  }

  // Default size is a bootstrap;
  // resized in `apply_camera_targets`
  // from window.
  let mut image = Image::new_fill(
    Extent3d {
      width:                 1280,
      height:                720,
      depth_or_array_layers: 1
    },
    TextureDimension::D2,
    &[0, 0, 0, 255],
    TextureFormat::Rgba8UnormSrgb,
    RenderAssetUsages::default()
  );
  image.texture_descriptor.usage =
    bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
      | bevy::render::render_resource::TextureUsages::COPY_DST
      | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT;

  let handle = images.add(image);
  commands.insert_resource(
    RenderToTextureRes {
      image: handle
    }
  );
}

#[instrument(level = "debug", skip_all)]
fn apply_camera_targets(
  effects: Option<
    Res<EffectsConfigRes>
  >,
  rtt: Option<Res<RenderToTextureRes>>,
  marker: Option<
    Res<PostProcessMarker>
  >,
  scene: Option<Res<SceneRes>>,
  cfg: Option<Res<ConfigRes>>,
  assets_root: Option<
    Res<AssetsRootRes>
  >,
  asset_server: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<
    Assets<PostProcessMaterial>
  >,
  mut images: ResMut<Assets<Image>>,
  windows: Query<&Window>,
  cameras: Query<(Entity, &Camera)>,
  mut commands: Commands
) {
  let Some(effects) = effects else {
    return;
  };
  if !effects.enabled {
    return;
  }
  let Some(rtt) = rtt else {
    return;
  };

  // Validate that the global effect
  // WGSL (if any) resolves from TOML,
  // so we can surface actionable
  // errors early.
  if let (
    Some(scene),
    Some(cfg),
    Some(assets_root)
  ) = (
    scene.as_deref(),
    cfg.as_deref(),
    assets_root.as_deref()
  ) {
    if let Some(global_id) = effects
      .global_effect_id
      .as_deref()
    {
      if let Some(list) =
        &scene.0.scene.effects
      {
        if let Some(effect) = list
          .iter()
          .find(|e| e.id == global_id)
        {
          // Kick load + parse
          // validation.
          if let Some(wgsl) = &effect.wgsl {
            let _ = flatfekt_assets::resolve::bevy_load::load_wgsl_shader(
              &asset_server,
              &cfg.0,
              &assets_root.0,
              wgsl,
            );
          } else if let Some(glsl) = &effect.glsl {
            let _ = flatfekt_assets::resolve::bevy_load::load_glsl_shader(
              &asset_server,
              &cfg.0,
              &assets_root.0,
              glsl,
            );
          }
        }
      }
    }
  }

  // If there are no cameras yet, do
  // nothing (instantiate_scene will
  // create them).
  if cameras.is_empty() {
    return;
  }

  let primary = windows
    .iter()
    .next()
    .map(|w| (w.width(), w.height()));

  if let Some((w, h)) = primary
    && let Some(img) =
      images.get_mut(&rtt.image)
  {
    let mut new_w =
      (w * effects.rtt_scale).round()
        as u32;
    let mut new_h =
      (h * effects.rtt_scale).round()
        as u32;
    new_w = new_w.max(1);
    new_h = new_h.max(1);

    let current =
      img.texture_descriptor.size;
    if current.width != new_w
      || current.height != new_h
    {
      *img = Image::new_fill(
        Extent3d {
          width:                 new_w,
          height:                new_h,
          depth_or_array_layers: 1
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default()
      );
      img.texture_descriptor.usage =
        bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
          | bevy::render::render_resource::TextureUsages::COPY_DST
          | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT;
    }
  }

  // Ensure the main camera (order 0)
  // renders into the offscreen image.
  for (e, cam) in cameras.iter() {
    if cam.order == 0 {
      commands.entity(e).insert(
        RenderTarget::Image(
          rtt.image.clone().into()
        )
      );
    }
  }

  // Spawn a fullscreen quad + a second
  // camera if we haven't already.
  if marker.is_none() {
    commands.insert_resource(
      PostProcessMarker
    );

    let mesh = meshes
      .add(Rectangle::new(2.0, 2.0));
    let mesh2d = Mesh2d(mesh);

    let mut intensity = 1.0;
    let mut shader_handle: Handle<Shader> =
      asset_server.load("flatfekt/shaders/postprocess.wgsl");

    if let (
      Some(scene),
      Some(global_id)
    ) = (
      scene.as_deref(),
      effects
        .global_effect_id
        .as_deref()
    ) {
      if let Some(list) =
        &scene.0.scene.effects
      {
        if let Some(effect) = list
          .iter()
          .find(|e| e.id == global_id)
        {
          if let Some(params) =
            &effect.params
          {
            if let Some(v) = params
              .get("intensity")
              .and_then(|v| {
                v.as_float()
              })
            {
              intensity = v as f32;
            }
          }

          if let Some(wgsl) = &effect.wgsl {
            shader_handle = flatfekt_assets::resolve::bevy_load::load_wgsl_shader(
              &asset_server,
              &cfg.as_deref().unwrap().0,
              &assets_root.as_deref().unwrap().0,
              wgsl,
            ).unwrap_or(shader_handle);
          } else if let Some(glsl) = &effect.glsl {
            shader_handle = flatfekt_assets::resolve::bevy_load::load_glsl_shader(
              &asset_server,
              &cfg.as_deref().unwrap().0,
              &assets_root.as_deref().unwrap().0,
              glsl,
            ).unwrap_or(shader_handle);
          }
        }
      }
    }

    let mat = materials.add(
      PostProcessMaterial {
        source: rtt.image.clone(),
        params: PostProcessParams {
          intensity
        },
        shader: shader_handle
      }
    );

    commands.spawn((
      mesh2d,
      MeshMaterial2d(mat),
      Transform::from_translation(
        Vec3::new(0.0, 0.0, 0.0)
      )
    ));

    // Second camera renders to window;
    // first camera target is set in
    // instantiate_scene.
    commands.spawn((
      Camera2d,
      Camera {
        order: 1,
        ..default()
      },
      RenderTarget::Window(
        bevy::window::WindowRef::Primary,
      ),
    ));
  }
}

#[derive(Resource)]
struct PostProcessMarker;
