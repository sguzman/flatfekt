use std::path::{
  Component,
  Path,
  PathBuf
};

use flatfekt_config::RootConfig;
use flatfekt_schema::AssetRef;
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum AssetResolveError {
  #[error(
    "asset root directory not \
     configured"
  )]
  MissingAssetsDir,

  #[error(
    "asset id indirection not \
     implemented: {0}"
  )]
  UnsupportedId(String),

  #[error(
    "asset path must be relative: {0}"
  )]
  AbsolutePath(String),

  #[error(
    "asset path contains parent \
     traversal '..': {0}"
  )]
  ParentTraversal(String)
}

#[instrument(level = "info", skip_all)]
pub fn assets_root(
  cfg: &RootConfig
) -> Result<PathBuf, AssetResolveError>
{
  cfg.app
    .as_ref()
    .and_then(|a| a.assets_dir.clone())
    .ok_or(AssetResolveError::MissingAssetsDir)
}

#[instrument(level = "info", skip_all)]
pub fn resolve_asset_path(
  root: &Path,
  asset: &AssetRef
) -> Result<PathBuf, AssetResolveError>
{
  let rel = match asset {
    | AssetRef::Path {
      path
    } => path,
    | AssetRef::Id {
      id
    } => {
      return Err(AssetResolveError::UnsupportedId(
        id.clone()
      ));
    }
    | AssetRef::String(s) => {
      Path::new(s)
    }
  };

  if rel.is_absolute() {
    return Err(
      AssetResolveError::AbsolutePath(
        rel.display().to_string()
      )
    );
  }

  for c in rel.components() {
    if matches!(c, Component::ParentDir)
    {
      return Err(AssetResolveError::ParentTraversal(
        rel.display().to_string()
      ));
    }
  }

  Ok(root.join(rel))
}

#[cfg(feature = "bevy")]
pub mod bevy_load {
  use std::path::Path;

  use bevy::asset::AssetServer;
  use bevy::prelude::{
    Font,
    Handle,
    Image
  };
  use flatfekt_schema::AssetRef;
  use tracing::instrument;

  use super::{
    AssetResolveError,
    resolve_asset_path
  };

  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn load_image(
    assets: &AssetServer,
    root: &Path,
    image: &AssetRef
  ) -> Result<
    Handle<Image>,
    AssetResolveError
  > {
    let abs =
      resolve_asset_path(root, image)?;
    let rel = abs
      .strip_prefix(root)
      .unwrap_or(&abs)
      .to_string_lossy()
      .to_string();
    Ok(assets.load(rel))
  }

  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn load_font(
    assets: &AssetServer,
    root: &Path,
    font: &AssetRef
  ) -> Result<
    Handle<Font>,
    AssetResolveError
  > {
    let abs =
      resolve_asset_path(root, font)?;
    let rel = abs
      .strip_prefix(root)
      .unwrap_or(&abs)
      .to_string_lossy()
      .to_string();
    Ok(assets.load(rel))
  }
}
