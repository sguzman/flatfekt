use std::collections::HashMap;
use std::fs;
use std::path::{
  Component,
  Path,
  PathBuf
};
use std::time::Instant;

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

  #[error("unknown asset id: {0}")]
  UnknownId(String),

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

#[derive(Debug, Clone)]
pub struct AssetMeta {
  pub bytes:       Option<u64>,
  pub modified:
    Option<std::time::SystemTime>,
  pub resolved_at: Instant
}

#[derive(Debug, Clone)]
pub struct ResolvedAsset {
  pub abs:  PathBuf,
  pub meta: AssetMeta
}

#[derive(Default)]
pub struct AssetCache {
  // Key is the serialized AssetRef
  // (stable enough for v0.1).
  pub resolved:
    HashMap<String, ResolvedAsset>,
  pub hits:     u64,
  pub misses:   u64
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

#[instrument(level = "info", skip_all)]
pub fn resolve_asset_path_cfg(
  cfg: &RootConfig,
  root: &Path,
  asset: &AssetRef
) -> Result<PathBuf, AssetResolveError>
{
  let rel: &Path =
    match asset {
      | AssetRef::Path {
        path
      } => path.as_path(),
      | AssetRef::String(s) => {
        Path::new(s)
      }
      | AssetRef::Id {
        id
      } => cfg
        .asset_path_for_id(id)
        .map(|p| p.as_path())
        .ok_or_else(|| {
          AssetResolveError::UnknownId(
            id.clone()
          )
        })?
    };

  resolve_asset_path(
    root,
    &AssetRef::Path {
      path: rel.to_path_buf()
    }
  )
}

#[instrument(level = "info", skip_all)]
pub fn resolve_cached(
  cache: &mut AssetCache,
  cfg: &RootConfig,
  root: &Path,
  asset: &AssetRef
) -> Result<
  ResolvedAsset,
  AssetResolveError
> {
  let key = asset_key(asset);
  if let Some(r) =
    cache.resolved.get(&key)
  {
    cache.hits += 1;
    tracing::debug!(
      key,
      "asset cache hit"
    );
    return Ok(r.clone());
  }

  cache.misses += 1;
  tracing::debug!(
    key,
    "asset cache miss"
  );
  let abs = resolve_asset_path_cfg(
    cfg, root, asset
  )?;

  let (bytes, modified) =
    match fs::metadata(&abs) {
      | Ok(md) => {
        (
          Some(md.len()),
          md.modified().ok()
        )
      }
      | Err(_) => (None, None)
    };

  let resolved = ResolvedAsset {
    abs:  abs.clone(),
    meta: AssetMeta {
      bytes,
      modified,
      resolved_at: Instant::now()
    }
  };
  cache
    .resolved
    .insert(key, resolved.clone());
  Ok(resolved)
}

fn asset_key(
  asset: &AssetRef
) -> String {
  match asset {
    | AssetRef::Path {
      path
    } => {
      format!("path:{}", path.display())
    }
    | AssetRef::Id {
      id
    } => format!("id:{id}"),
    | AssetRef::String(s) => {
      format!("string:{s}")
    }
  }
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
  use flatfekt_config::RootConfig;
  use flatfekt_schema::AssetRef;
  use tracing::instrument;

  use super::{
    AssetCache,
    AssetResolveError,
    resolve_asset_path_cfg,
    resolve_cached
  };

  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn load_image(
    assets: &AssetServer,
    cfg: &RootConfig,
    root: &Path,
    image: &AssetRef
  ) -> Result<
    Handle<Image>,
    AssetResolveError
  > {
    let abs = resolve_asset_path_cfg(
      cfg, root, image
    )?;
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
    cfg: &RootConfig,
    root: &Path,
    font: &AssetRef
  ) -> Result<
    Handle<Font>,
    AssetResolveError
  > {
    let abs = resolve_asset_path_cfg(
      cfg, root, font
    )?;
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
  pub fn load_image_cached(
    cache: &mut AssetCache,
    assets: &AssetServer,
    cfg: &RootConfig,
    root: &Path,
    image: &AssetRef
  ) -> Result<
    Handle<Image>,
    AssetResolveError
  > {
    let resolved = resolve_cached(
      cache, cfg, root, image
    )?;
    let rel = resolved
      .abs
      .strip_prefix(root)
      .unwrap_or(&resolved.abs)
      .to_string_lossy()
      .to_string();
    Ok(assets.load(rel))
  }

  #[instrument(
    level = "info",
    skip_all
  )]
  pub fn load_font_cached(
    cache: &mut AssetCache,
    assets: &AssetServer,
    cfg: &RootConfig,
    root: &Path,
    font: &AssetRef
  ) -> Result<
    Handle<Font>,
    AssetResolveError
  > {
    let resolved = resolve_cached(
      cache, cfg, root, font
    )?;
    let rel = resolved
      .abs
      .strip_prefix(root)
      .unwrap_or(&resolved.abs)
      .to_string_lossy()
      .to_string();
    Ok(assets.load(rel))
  }
}
